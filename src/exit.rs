use anyhow::{Context as _, Result, anyhow};
use once_cell::sync::OnceCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

static FLAG: OnceCell<Arc<AtomicBool>> = OnceCell::new();

#[derive(Clone)]
pub(crate) struct Exit;

impl Exit {
    pub(crate) fn trigger() {
        FLAG.get()
            .unwrap_or_else(|| {
                log::error!("exit handler is not set");
                std::process::exit(1);
            })
            .store(true, Ordering::Relaxed)
    }

    pub(crate) fn setup_handler() -> Result<()> {
        log::info!("Setting exit handler...");
        let flag = Arc::new(AtomicBool::new(false));
        println!("{:?}", std::thread::current().id());

        for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
            signal_hook::flag::register(signal, Arc::clone(&flag))
                .context("failed to set exit handler")?;
        }

        FLAG.set(flag)
            .map_err(|_| anyhow!("setup_handler() must be called once"))?;

        Ok(())
    }

    pub(crate) fn received() -> bool {
        FLAG.get()
            .unwrap_or_else(|| {
                log::error!("exit handler is not set");
                std::process::exit(1);
            })
            .load(Ordering::Relaxed)
    }
}
