mod reader;
mod writer;

use crate::{mpclipboard::MPClipboard, timer::TimerBased, tray::Tray};
pub(crate) use reader::ClipboardReader;
use std::ops::ControlFlow;
pub(crate) use writer::ClipboardWriter;

pub(crate) struct Clipboard {
    tray: Tray,
    reader: ClipboardReader,
}

impl Clipboard {
    pub(crate) fn new(tray: Tray) -> Self {
        Self {
            tray,
            reader: ClipboardReader::new(),
        }
    }
}

impl TimerBased for Clipboard {
    fn work(&mut self) -> ControlFlow<()> {
        if let Some(text) = self.reader.read() {
            self.tray.push_local(&text);
            MPClipboard::send(text);
        }
        ControlFlow::Continue(())
    }
}
