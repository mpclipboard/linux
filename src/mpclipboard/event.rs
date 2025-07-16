use anyhow::{Context as _, Result};
use mpclipboard_generic_client::Output;
use std::ffi::CString;

#[derive(Debug)]
pub(crate) struct MPClipboardEvent {
    pub(crate) text: Option<String>,
    pub(crate) connectivity: Option<bool>,
}

impl MPClipboardEvent {
    pub(crate) fn from_output(output: Output) -> Result<Option<Self>> {
        let text = if output.text.is_null() {
            None
        } else {
            let text = unsafe { CString::from_raw(output.text.cast()) };
            let text = text.to_str().context("non-utf-8 text")?.to_string();
            Some(text)
        };

        let connectivity = if output.connectivity.is_null() {
            None
        } else {
            let connectivity = unsafe { Box::from_raw(output.connectivity) };
            Some(*connectivity)
        };

        if text.is_none() && connectivity.is_none() {
            Ok(None)
        } else {
            Ok(Some(Self { text, connectivity }))
        }
    }
}
