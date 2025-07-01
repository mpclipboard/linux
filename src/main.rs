use crate::{
    exit::Exit, local_clipboard::LocalClipboard, mpclipboard::MPClipboard, timer::Timer, tray::Tray,
};
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

mod exit;
mod local_clipboard;
mod mpclipboard;
mod timer;
mod tray;

fn main() -> Result<()> {
    MPClipboard::start()?;
    let exit = Exit::new()?;
    let clipboard = LocalClipboard::new();
    let tray = {
        let exit = exit.clone();
        Tray::new(move || exit.trigger())
    }?;

    let mut timer = Timer::new(Duration::from_millis(100));

    timer.add(1, move || {
        if exit.received() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });

    timer.add(1, {
        let tray = tray.clone();
        let clipboard = clipboard.clone();
        move || {
            if let Some(event) = MPClipboard::recv() {
                if let Some(connectivity) = event.connectivity {
                    tray.set_connectivity(connectivity);
                }

                if let Some(text) = event.text {
                    clipboard.write(&text);
                    tray.push_received(&text);
                }
            }
            ControlFlow::Continue(())
        }
    });

    timer.add(10, move || {
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
        ControlFlow::Continue(())
    });
    timer.start()?;

    log::info!("exiting...");
    if let Err(err) = MPClipboard::stop() {
        log::error!("{err:?}");
    }

    Ok(())
}
