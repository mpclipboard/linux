use anyhow::{Context as _, Result};
use std::{cell::RefCell, io::Read as _, rc::Rc};

thread_local! {
    static LAST: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    static BUF: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(vec![0; 1_024]));
}

fn with_state<F, T>(f: F) -> T
where
    F: FnOnce(&mut Option<String>, &mut Vec<u8>) -> T,
{
    LAST.with(|last| {
        BUF.with(|buf| {
            let mut last = last.borrow_mut();
            let mut buf = buf.borrow_mut();
            f(&mut last, &mut buf)
        })
    })
}

pub(crate) struct Clipboard;

impl Clipboard {
    pub(crate) fn read() -> Option<String> {
        with_state(|last, buf| {
            let text = match read_text(buf) {
                Ok(Some(text)) => text,
                Ok(None) => return None,
                Err(err) => {
                    log::error!("{err:?}");
                    return None;
                }
            };

            if last.as_ref().is_some_and(|v| v == &text) {
                None
            } else {
                let text = text.to_string();
                *last = Some(text.clone());
                Some(text)
            }
        })
    }

    pub(crate) fn write(text: &str) {
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
