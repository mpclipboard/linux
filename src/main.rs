use crate::{
    exit_handler::ExitHandler, local_clipboard::LocalClipboard, mpclipboard::MPClipboard,
    timer::Timer, tray::Tray,
};
use anyhow::{Context as _, Result};
use ksni::blocking::TrayMethods;

mod exit_handler;
mod local_clipboard;
mod mpclipboard;
mod timer;
mod tray;

fn main() -> Result<()> {
    MPClipboard::start()?;
    let exit_handler = ExitHandler::new()?;
    let mut clipboard = LocalClipboard::new();
    let tray = Tray::new(exit_handler.clone())
        .spawn()
        .context("failed to spawn Tray")?;
    let mut timer = Timer::new();

    while exit_handler.keep_running() {
        if timer.passed(10) {
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
        }

        timer.tick(100);
    }

    log::info!("exiting...");
    if let Err(err) = MPClipboard::stop() {
        log::error!("{err:?}");
    }

    Ok(())
}
