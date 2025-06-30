use anyhow::{Context as _, Result, ensure};
use mpclipboard_generic_client::{
    Output, mpclipboard_config_read_from_xdg_config_dir, mpclipboard_poll, mpclipboard_send,
    mpclipboard_setup, mpclipboard_start_thread, mpclipboard_stop_thread,
};
use std::ffi::CString;

pub(crate) struct MPClipboard;

#[derive(Debug)]
pub(crate) struct MPClipboardEvent {
    pub(crate) text: Option<String>,
    pub(crate) connectivity: Option<bool>,
}

impl MPClipboardEvent {
    fn from_output(output: Output) -> Result<Option<Self>> {
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

impl MPClipboard {
    pub(crate) fn start() -> Result<()> {
        mpclipboard_setup();
        let config = mpclipboard_config_read_from_xdg_config_dir();
        ensure!(
            !config.is_null(),
            "failed to construct a config from XDG dir"
        );
        mpclipboard_start_thread(config);

        Ok(())
    }

    pub(crate) fn stop() -> Result<()> {
        ensure!(
            mpclipboard_stop_thread(),
            "failed to stop mpclipboard thread"
        );
        Ok(())
    }

    pub(crate) fn send(text: String) {
        let Ok(text) = CString::new(text) else {
            log::error!("failed to convert text to C string");
            return;
        };
        mpclipboard_send(text.as_ptr().cast());
    }

    pub(crate) fn recv() -> Option<MPClipboardEvent> {
        let output = mpclipboard_poll();
        match MPClipboardEvent::from_output(output) {
            Ok(event) => event,
            Err(err) => {
                log::error!("{err:?}");
                std::process::exit(1)
            }
        }
    }
}
