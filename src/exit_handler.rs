use anyhow::{Context as _, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub(crate) struct ExitHandler {
    exit: Arc<AtomicBool>,
}

impl ExitHandler {
    pub(crate) fn trigger_manually(&self) {
        self.exit.store(false, Ordering::Relaxed);
    }

    pub(crate) fn new() -> Result<Self> {
        let flag = Arc::new(AtomicBool::new(false));
        log::info!("Setting exit handler...");

        for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
            signal_hook::flag::register(signal, Arc::clone(&flag))
                .context("failed to set exit handler")?;
        }

        Ok(Self { exit: flag })
    }

    pub(crate) fn keep_running(&self) -> bool {
        !self.exit.load(Ordering::Relaxed)
    }
}
