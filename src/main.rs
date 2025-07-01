use crate::{
    exit::Exit, local_clipboard::LocalClipboard, mpclipboard::MPClipboard, timer::Timer, tray::Tray,
};
use anyhow::Result;

mod exit;
mod local_clipboard;
mod mpclipboard;
mod timer;
mod tray;

fn main() -> Result<()> {
    MPClipboard::start()?;
    let exit = Exit::new()?;
    let mut clipboard = LocalClipboard::new();
    let tray = {
        let exit = exit.clone();
        Tray::new(move || exit.trigger())
    }?;
    let mut timer = Timer::new();

    while exit.received() {
        if timer.passed(10) {
            if let Some(text) = clipboard.read() {
                tray.push_local(&text);
                MPClipboard::send(text);
            }
            if let Some(event) = MPClipboard::recv() {
                if let Some(connectivity) = event.connectivity {
                    tray.set_connectivity(connectivity);
                }

                if let Some(text) = event.text {
                    clipboard.write(&text);
                    tray.push_received(&text);
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
