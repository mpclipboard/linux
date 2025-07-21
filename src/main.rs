use crate::{
    clipboard::{LocalReader, LocalWriter},
    mpclipboard::MPClipboard,
    tray::Tray,
};
use anyhow::{Context as _, Result};
use mpclipboard_generic_client::Event as MPClipboardEvent;
use tokio::signal::unix::SignalKind;
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

    mpclipboard.stop().await?;
    tray.stop().await;
    listener.wait().await?;

    Ok(())
}
