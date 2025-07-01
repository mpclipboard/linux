use anyhow::{Context as _, Result};
use std::io::Read as _;

pub(crate) struct LocalClipboard {
    last: Option<String>,
    buf: Vec<u8>,
}

impl LocalClipboard {
    pub(crate) fn new() -> Self {
        Self {
            last: None,
            buf: vec![0; 1_024],
        }
    }

    pub(crate) fn read(&mut self) -> Option<String> {
        let text = match read_text(&mut self.buf) {
            Ok(text) => text?,
            Err(err) => {
                log::error!("{err:?}");
                return None;
            }
        };

        if self.last.as_ref().is_some_and(|v| v == text) {
            return None;
        }

        let text = text.to_string();
        self.last = Some(text.clone());

        Some(text)
    }

    pub(crate) fn write(&self, text: &str) {
        if let Err(err) = write_text(text) {
            log::error!("{err:?}");
        }
    }
}

fn read_text(buf: &mut Vec<u8>) -> Result<Option<&str>> {
    use wl_clipboard_rs::{
        paste::{ClipboardType, Error as PasteError, MimeType, Seat, get_contents},
        utils::is_text,
    };

    let clipboard = ClipboardType::Regular;
    let seat = Seat::Unspecified;
    let mime_type = MimeType::Text;

    let (mut reader, mime) = match get_contents(clipboard, seat, mime_type) {
        Ok(data) => data,
        Err(PasteError::NoSeats | PasteError::ClipboardEmpty | PasteError::NoMimeType) => {
            return Ok(None);
        }
        Err(err) => {
            let err = anyhow::Error::from(err).context("failed to get clipboard contents");
            return Err(err);
        }
    };

    if !is_text(&mime) {
        return Ok(None);
    }

    buf.clear();
    let len = reader
        .read_to_end(buf)
        .context("failed to read clipboard contents")?;

    let text = std::str::from_utf8(&buf[..len]).context("non-utf-8 clipboard contents")?;

    if text.contains('\0') {
        return Ok(None);
    }

    Ok(Some(text))
}

fn write_text(text: &str) -> Result<()> {
    use wl_clipboard_rs::copy::{MimeType, Options, Source, copy};
    let options = Options::default();
    copy(
        options,
        Source::Bytes(text.to_string().into_bytes().into_boxed_slice()),
        MimeType::Text,
    )
    .context("failed to write contents to clipboard")?;
    Ok(())
}
