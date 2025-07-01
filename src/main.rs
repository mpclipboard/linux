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
    let tray = Tray::new()?;

    let mut timer = Timer::new(Duration::from_millis(100));

    timer.add(1, move || {
        if Exit::received() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });

    timer.add(1, {
        let tray = tray.clone();
        move || {
            if let Some(event) = MPClipboard::recv() {
                if let Some(connectivity) = event.connectivity {
                    tray.set_connectivity(connectivity);
                }

                if let Some(text) = event.text {
                    Clipboard::write(&text);
                    tray.push_received(&text);
                }
            }
            ControlFlow::Continue(())
        }
    });

    timer.add(10, move || {
        if let Some(text) = Clipboard::read() {
            tray.push_local(&text);
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
