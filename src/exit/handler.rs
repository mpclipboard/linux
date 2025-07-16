use anyhow::{Context as _, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub(crate) struct ExitHandler {
    flag: Arc<AtomicBool>,
}

impl ExitHandler {
    pub(crate) fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn trigger(&self) {
        self.flag.store(true, Ordering::Relaxed)
    }

    pub(crate) fn setup_handler(&self) -> Result<()> {
        log::info!("Setting exit handler...");

        for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
            signal_hook::flag::register(signal, Arc::clone(&self.flag))
                .context("failed to set exit handler")?;
        }

        Ok(())
    }

    pub(crate) fn received(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}
