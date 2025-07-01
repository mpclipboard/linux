mod buffer;
mod event;
mod state;

use anyhow::{Context as _, Result};
use ksni::blocking::{Handle, TrayMethods};
use state::TrayState;

#[derive(Clone)]
pub(crate) struct Tray {
    handle: Handle<TrayState>,
}

impl Tray {
    pub(crate) fn new(exit: impl Fn() + Send + Sync + 'static) -> Result<Self> {
        let state = TrayState::new(exit);
        let handle = state.spawn().context("failed to spawn Tray")?;
        Ok(Self { handle })
    }

    pub(crate) fn push_local(&self, text: &str) {
        self.handle.update(|tray| tray.push_local(text));
    }

    pub(crate) fn push_received(&self, text: &str) {
        self.handle.update(|tray| tray.push_received(text));
    }

    pub(crate) fn set_connectivity(&self, connectivity: bool) {
        self.handle
            .update(|tray| tray.set_connectivity(connectivity));
    }
}
