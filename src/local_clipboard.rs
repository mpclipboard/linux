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

    pub(crate) fn read(&mut self) -> Result<Option<String>> {
        use wl_clipboard_rs::{
            paste::{ClipboardType, Error as PasteError, MimeType, Seat, get_contents},
            utils::is_text,
        };

        let (mut reader, mime) =
            match get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text) {
                Ok(data) => data,
                Err(PasteError::NoSeats | PasteError::ClipboardEmpty | PasteError::NoMimeType) => {
                    return Ok(None);
                }
                Err(err) => {
                    return Err(anyhow::Error::new(err).context("failed to get clipboard contents"));
                }
            };

        if !is_text(&mime) {
            return Ok(None);
        }

        self.buf.clear();
        let len = reader
            .read_to_end(&mut self.buf)
            .context("failed to read clipboard contents")?;

        let text = std::str::from_utf8(&self.buf[..len]).context("non-utf-8 clipboard contents")?;
        if text.contains('\0') {
            return Ok(None);
        }

        if self.last.as_ref().is_some_and(|v| v == text) {
            return Ok(None);
        }

        let text = text.to_string();
        self.last = Some(text.clone());

        Ok(Some(text))
    }

    pub(crate) fn write(&mut self, text: &str) -> Result<()> {
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
}
