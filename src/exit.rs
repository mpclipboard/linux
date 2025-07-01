use anyhow::{Context as _, Result};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub(crate) struct Exit {
    flag: Arc<AtomicBool>,
}

impl Exit {
    pub(crate) fn trigger(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub(crate) fn new() -> Result<Self> {
        let flag = Arc::new(AtomicBool::new(false));
        log::info!("Setting exit handler...");

        for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
            signal_hook::flag::register(signal, Arc::clone(&flag))
                .context("failed to set exit handler")?;
        }

        Ok(Self { flag })
    }

    pub(crate) fn received(&self) -> bool {
        !self.flag.load(Ordering::Relaxed)
    }
}
