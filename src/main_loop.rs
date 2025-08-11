use std::time::Duration;

use crate::{
    clipboard::{LocalReader, LocalWriter},
    mpclipboard::MPClipboard,
    tray::Tray,
};
use anyhow::{Context as _, Result};
use mpclipboard_generic_client::{Clip, Event, Store};
use tokio::{
    signal::unix::{Signal, SignalKind},
    time::timeout,
};
use tokio_util::sync::CancellationToken;

pub(crate) struct MainLoop {
    token: CancellationToken,
    mpclipboard: MPClipboard,
    tray: Tray,
    clipboard: LocalReader,
    store: Store,
    sigterm: Signal,
    sigint: Signal,
}

impl MainLoop {
    pub(crate) async fn new() -> Result<Self> {
        let token = CancellationToken::new();
        let mpclipboard = MPClipboard::start()?;
        let tray = Tray::spawn(token.clone()).await?;
        let clipboard = LocalReader::spawn(token.clone()).await;
        let sigterm = tokio::signal::unix::signal(SignalKind::terminate())
            .context("failed to construct SIGTERM handler")?;
        let sigint = tokio::signal::unix::signal(SignalKind::interrupt())
            .context("failed to construct SIGINT handler")?;
        let store = Store::new();

        Ok(Self {
            token,
            mpclipboard,
            tray,
            clipboard,
            sigterm,
            sigint,
            store,
        })
    }

    pub(crate) async fn start(mut self) {
        loop {
            tokio::select! {
                events = self.mpclipboard.recv() => {
                    match events {
                        Ok(events) => {
                            self.on_mpclipboard_events(events).await;
                        },
                        Err(err) => {
                            log::info!("MPClipboard thread has crashes, exiting: {err:?}");
                            break;
                        }
                    }
                }

                Some(text) = self.clipboard.recv() => {
                    self.on_text_from_local_clipboard(text).await;
                }

                _ = self.sigterm.recv() => self.on_signal("SIGTERM"),
                _ = self.sigint.recv() => self.on_signal("SIGINT"),

                _ = self.token.cancelled() => {
                    log::info!("exiting...");
                    break;
                }
            }
        }

        self.stop().await;
    }

    async fn on_mpclipboard_events(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::ConnectivityChanged(connectivity) => {
                    log::info!(target: "MPClipboard", "connectivity = {connectivity}");
                    self.tray.set_connectivity(connectivity).await;
                }
                Event::NewClip(clip) => {
                    log::info!(target: "MPClipboard", "new clip {clip:?}");
                    if self.store.add(&clip) {
                        LocalWriter::write(&clip.text);
                        self.tray.push_received(&clip.text).await;
                    }
                }
            }
        }
    }

    async fn on_text_from_local_clipboard(&mut self, text: String) {
        log::info!(target: "LocalReader", "{text}");
        let clip = Clip::new(&text);
        if self.store.add(&clip) {
            if let Err(err) = self.mpclipboard.send(clip) {
                log::error!("failed to send text to MPClipboard server: {err:?}");
            }
            self.tray.push_sent(&text).await;
        }
    }

    fn on_signal(&self, signal: &str) {
        log::info!("{signal} received...");
        self.token.cancel();
    }

    pub(crate) async fn stop(self) {
        if let Err(err) = self.mpclipboard.stop() {
            log::error!("failed to stop mpclipboard thread: {err:?}");
        }

        if let Err(_) = timeout(Duration::from_secs(5), self.tray.stop()).await {
            log::warn!("Tray shutdown timed out after 5 seconds");
        }
        if let Err(_) = timeout(Duration::from_secs(5), self.clipboard.wait()).await {
            log::warn!("LocalReader shutdown timed out after 5 seconds");
        }
    }
}
