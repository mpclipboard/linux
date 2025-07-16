mod bindings;
mod event;

use crate::{clipboard::ClipboardWriter, timer::TimerBased, tray::Tray};
pub(crate) use bindings::MPClipboard;
use std::ops::ControlFlow;

pub(crate) struct MPClipboardActor {
    tray: Tray,
}

impl MPClipboardActor {
    pub(crate) fn new(tray: Tray) -> Self {
        Self { tray }
    }
}

impl TimerBased for MPClipboardActor {
    fn work(&mut self) -> ControlFlow<()> {
        if let Some(event) = MPClipboard::poll() {
            if let Some(connectivity) = event.connectivity {
                self.tray.set_connectivity(connectivity);
            }

            if let Some(text) = event.text {
                ClipboardWriter::write(&text);
                self.tray.push_received(&text);
            }
        }
        ControlFlow::Continue(())
    }
}
