use crate::{
    clipboard::{LocalReader, LocalWriter},
    mpclipboard::MPClipboard,
    tray::Tray,
};
use anyhow::{Context as _, Result};
use mpclipboard_generic_client::Event as MPClipboardEvent;
use std::time::Duration;
use tokio::{signal::unix::SignalKind, time::timeout};
use tokio_util::sync::CancellationToken;

mod clipboard;
mod mpclipboard;
mod tray;

#[tokio::main]
async fn main() -> Result<()> {
    let token = CancellationToken::new();

    let mut mpclipboard = MPClipboard::spawn();
    let tray = Tray::spawn(token.clone()).await?;
    let mut listener = LocalReader::spawn(token.clone()).await;

    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
        .context("failed to construct SIGTERM handler")?;
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
        .context("failed to construct SIGINT handler")?;

    loop {
        tokio::select! {
            Some(event) = mpclipboard.recv() => {
                log::info!(target: "MPClipboard", "{event:?}");

                match event {
                    MPClipboardEvent::ConnectivityChanged(connectivity) => {
                        tray.set_connectivity(connectivity).await;
                    }
                    MPClipboardEvent::NewClip(clip) => {
                        LocalWriter::write(&clip.text);
                        tray.push_received(&clip.text).await;
                    }
                }
            }

            Some(text) = listener.recv() => {
                log::info!(target: "LocalReader", "{text}");
                mpclipboard.send(&text).await?;
                tray.push_sent(&text).await;
            }

            _ = sigterm.recv() => {
                log::info!("SIGTERM received...");
                token.cancel();
            }
            _ = sigint.recv() => {
                log::info!("SIGINT received...");
                token.cancel();
            }

            _ = token.cancelled() => {
                log::info!("exiting...");
                break;
            }
        }
    }

    if let Err(_) = timeout(Duration::from_secs(5), mpclipboard.stop()).await {
        log::warn!("MPClipboard shutdown timed out after 5 seconds");
    }
    if let Err(_) = timeout(Duration::from_secs(5), tray.stop()).await {
        log::warn!("Tray shutdown timed out after 5 seconds");
    }
    if let Err(_) = timeout(Duration::from_secs(5), listener.wait()).await {
        log::warn!("LocalReader shutdown timed out after 5 seconds");
    }

    Ok(())
}
