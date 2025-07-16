use crate::mpclipboard::event::MPClipboardEvent;
use mpclipboard_generic_client::{
    mpclipboard_config_read_from_xdg_config_dir, mpclipboard_poll, mpclipboard_send,
    mpclipboard_setup, mpclipboard_start_thread, mpclipboard_stop_thread,
};
use std::ffi::CString;

pub(crate) struct MPClipboard;

impl MPClipboard {
    pub(crate) fn setup() {
        mpclipboard_setup();
    }

    pub(crate) fn start() {
        let config = mpclipboard_config_read_from_xdg_config_dir();
        if config.is_null() {
            log::error!("failed to construct a config from XDG dir");
            std::process::exit(1);
        }
        mpclipboard_start_thread(config);
    }

    pub(crate) fn stop() {
        if !mpclipboard_stop_thread() {
            log::error!("failed to stop MPClipboard thread")
        }
    }

    pub(crate) fn send(text: String) {
        let Ok(text) = CString::new(text) else {
            log::error!("failed to convert text to C string");
            return;
        };
        mpclipboard_send(text.as_ptr().cast());
    }

    pub(crate) fn poll() -> Option<MPClipboardEvent> {
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
