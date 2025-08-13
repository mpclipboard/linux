use std::time::Duration;

use crate::{
    clipboard::{LocalReader, LocalWriter},
    mpclipboard::MPClipboard,
    tray::Tray,
};
use anyhow::{Context as _, Result};
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

        Ok(Self {
            token,
            mpclipboard,
            tray,
            clipboard,
            sigterm,
            sigint,
        })
    }

    pub(crate) async fn start(mut self) {
        loop {
            tokio::select! {
                readable = self.mpclipboard.readable() => {
                    if let Err(err) = readable {
                        log::info!("MPClipboard thread has crashes, exiting: {err:?}");
                        break;
                    }
                    self.recv_from_mpclipboard().await;
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

    async fn recv_from_mpclipboard(&mut self) {
        let (text, connectivity) = match self.mpclipboard.recv().await {
            Ok(pair) => pair,
            Err(err) => {
                log::error!("{err:?}");
                return;
            }
        };

        if let Some(connectivity) = connectivity {
            log::info!(target: "MPClipboard", "connectivity = {connectivity}");
            self.tray.set_connectivity(connectivity).await;
        }

        if let Some(text) = text {
            log::info!(target: "MPClipboard", "new clip {text:?}");
            LocalWriter::write(&text);
            self.tray.push_received(&text).await;
        }
    }

    async fn on_text_from_local_clipboard(&mut self, text: String) {
        log::info!(target: "LocalReader", "{text}");
        match self.mpclipboard.send(&text).await {
            Ok(true) => {
                self.tray.push_sent(&text).await;
            }
            Ok(false) => {}
            Err(err) => {
                log::error!("failed to send text to MPClipboard server: {err:?}");
            }
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

        if timeout(Duration::from_secs(5), self.tray.stop())
            .await
            .is_err()
        {
            log::warn!("Tray shutdown timed out after 5 seconds");
        }
        if timeout(Duration::from_secs(5), self.clipboard.wait())
            .await
            .is_err()
        {
            log::warn!("LocalReader shutdown timed out after 5 seconds");
        }
    }
}
