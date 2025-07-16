use crate::{
    clipboard::Clipboard,
    exit::ExitActor,
    mpclipboard::{MPClipboard, MPClipboardActor},
    timer::Timer,
    tray::Tray,
};
use anyhow::Result;
use std::time::Duration;

mod clipboard;
mod exit;
mod mpclipboard;
mod timer;
mod tray;

fn main() -> Result<()> {
    let exit = ExitActor::new();
    exit.handler().setup_handler()?;

    let tray = Tray::spawn(exit.handler())?;

    MPClipboard::setup();
    MPClipboard::start();
    let mpclipboard = MPClipboardActor::new(tray.clone());

    let clipboard = Clipboard::new(tray.clone());

    let mut timer = Timer::new(Duration::from_millis(100));

    timer.add(Duration::from_millis(100), exit);
    timer.add(Duration::from_millis(100), mpclipboard);
    timer.add(Duration::from_secs(1), clipboard);

    timer.start()?;
    log::info!("exiting...");
    MPClipboard::stop();

    Ok(())
}
