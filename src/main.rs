use crate::{
    exit_handler::ExitHandler, local_clipboard::LocalClipboard, mpclipboard::MPClipboard,
    tray::Tray,
};
use anyhow::{Context as _, Result};
use ksni::blocking::TrayMethods;

mod exit_handler;
mod local_clipboard;
mod mpclipboard;
mod tray;

fn main() -> Result<()> {
    MPClipboard::start()?;
    let exit_handler = ExitHandler::new()?;
    let mut clipboard = LocalClipboard::new();
    let tray = Tray::new(exit_handler.clone())
        .spawn()
        .context("failed to spawn Tray")?;

    while exit_handler.keep_running() {
        if let Some(text) = clipboard.read()? {
            tray.update(|tray| tray.push_local(&text));
            MPClipboard::send(text);
        }
        if let Some(event) = MPClipboard::recv() {
            if let Some(connectivity) = event.connectivity {
                tray.update(|tray| tray.set_connectivity(connectivity));
            }

            if let Some(text) = event.text {
                clipboard.write(&text)?;
                tray.update(|tray| tray.push_received(&text));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    log::info!("exiting...");
    if let Err(err) = MPClipboard::stop() {
        log::error!("{err:?}");
    }

    Ok(())
}
