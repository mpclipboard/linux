mod buffer;
mod event;
mod state;

use std::{cell::RefCell, rc::Rc};

use anyhow::{Context as _, Result};
use ksni::blocking::{Handle, TrayMethods};
use state::TrayState;

thread_local! {
    static HANDLE: Rc<RefCell<Option<Handle<TrayState>>>> = Rc::new(RefCell::new(None));
}
fn update<F>(f: F)
where
    F: FnOnce(&mut TrayState),
{
    HANDLE.with(|handle| {
        if let Some(handle) = handle.borrow().as_ref() {
            handle.update(f);
        } else {
            log::error!("Tray handle is not set, did you call Tray::spawn() ?");
            std::process::exit(1);
        }
    })
}

#[derive(Clone)]
pub(crate) struct Tray;

impl Tray {
    pub(crate) fn spawn() -> Result<()> {
        let state = TrayState::new();
        let handle = state.spawn().context("failed to spawn Tray")?;
        HANDLE.with(|global| *global.borrow_mut() = Some(handle));
        Ok(())
    }

    pub(crate) fn push_local(text: &str) {
        update(|tray| tray.push_local(text));
    }

    pub(crate) fn push_received(text: &str) {
        update(|tray| tray.push_received(text));
    }

    pub(crate) fn set_connectivity(connectivity: bool) {
        update(|tray| tray.set_connectivity(connectivity));
    }
}
