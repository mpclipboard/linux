use anyhow::{Context as _, Result};
use mpclipboard_common::Clip;
use mpclipboard_generic_client::{Command, Config, Event, MainLoop, mpclipboard_setup};
use tokio::{
    sync::mpsc::{Receiver, Sender, channel},
    task::JoinHandle,
};

pub(crate) struct MPClipboard {
    ctx: Sender<Command>,
    erx: Receiver<Event>,
    exit_tx: Sender<()>,
    handle: JoinHandle<()>,
}

impl MPClipboard {
    pub(crate) fn spawn() -> Self {
        mpclipboard_setup();

        let config = match Config::read_from_xdg_config_dir() {
            Ok(config) => config,
            Err(err) => {
                log::error!(target: "MPClipboard", "failed to read a config from XDG dir: {err:?}");
                std::process::exit(1);
            }
        };
        let config = Box::leak(Box::new(config));

        let (ctx, crx) = channel(255);
        let (etx, erx) = channel(255);
        let (exit_tx, exit_rx) = channel(1);

        let handle = tokio::spawn(async move {
            let mut main_loop = MainLoop::new(crx, etx, exit_rx, config);
            if let Err(err) = main_loop.start().await {
                log::error!(target: "MPClipboard", "{err:?}");
            }
        });

        Self {
            ctx,
            erx,
            exit_tx,
            handle,
        }
    }

    pub(crate) async fn stop(self) -> Result<()> {
        self.exit_tx
            .send(())
            .await
            .context("failed to send MPClipboard exit signal")?;
        self.handle
            .await
            .context("failed to join MPClipboard task")?;
        Ok(())
    }

    pub(crate) async fn send(&self, text: &str) -> Result<()> {
        let command = Command::NewClip(Clip::new(text.to_string()));
        self.ctx
            .send(command)
            .await
            .context("failed to send command")?;
        Ok(())
    }

    pub(crate) async fn recv(&mut self) -> Option<Event> {
        self.erx.recv().await
    }
}
