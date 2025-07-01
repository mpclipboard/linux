use crate::{clipboard::Clipboard, exit::Exit, mpclipboard::MPClipboard, timer::Timer, tray::Tray};
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

mod clipboard;
mod exit;
mod mpclipboard;
mod timer;
mod tray;

fn main() -> Result<()> {
    MPClipboard::start()?;
    Exit::setup_handler()?;
    Tray::spawn()?;

    let mut timer = Timer::new(Duration::from_millis(100));

    timer.add(1, || {
        if Exit::received() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });

    timer.add(1, || {
        if let Some(event) = MPClipboard::recv() {
            if let Some(connectivity) = event.connectivity {
                Tray::set_connectivity(connectivity);
            }

            if let Some(text) = event.text {
                Clipboard::write(&text);
                Tray::push_received(&text);
            }
        }
        ControlFlow::Continue(())
    });

    timer.add(10, || {
        if let Some(text) = Clipboard::read() {
            Tray::push_local(&text);
            MPClipboard::send(text);
        }
        ControlFlow::Continue(())
    });

    timer.start()?;
    log::info!("exiting...");
    if let Err(err) = MPClipboard::stop() {
        log::error!("{err:?}");
    }

    Ok(())
}
