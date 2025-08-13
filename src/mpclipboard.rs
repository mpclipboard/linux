use anyhow::{Context as _, Result};
use mpclipboard_generic_client::{Config, ConfigReadOption, Handle, Logger, TLS, Thread};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use tokio::net::unix::pipe::Receiver;

pub(crate) struct MPClipboard {
    handle: Handle,
    receiver: Receiver,
}

const CONFIG_READ_OPTION: ConfigReadOption = if cfg!(debug_assertions) {
    ConfigReadOption::FromLocalFile
} else {
    ConfigReadOption::FromXdgConfigDir
};

impl MPClipboard {
    pub(crate) fn init() -> Result<()> {
        Logger::init();
        TLS::init()?;
        Ok(())
    }

    pub(crate) fn start() -> Result<Self> {
        let config = Config::read(CONFIG_READ_OPTION)?;
        let mut handle = Thread::start(config)?;
        let pipe_reader = handle
            .pipe_reader()
            .context("malformed Handle: no pipe_reader")?;

        let fd = pipe_reader.as_raw_fd();
        std::mem::forget(pipe_reader);
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let receiver = Receiver::from_owned_fd(fd)
            .context("failed to create tokio::net::unix::pipe::Receiver")?;

        Ok(Self { handle, receiver })
    }

    pub(crate) fn stop(self) -> Result<()> {
        self.handle.stop()
    }

    pub(crate) async fn send(&mut self, text: &str) -> Result<bool> {
        self.handle.send(text).await
    }

    pub(crate) async fn readable(&self) -> Result<()> {
        self.receiver.readable().await?;
        Ok(())
    }

    pub(crate) async fn recv(&mut self) -> Result<(Option<String>, Option<bool>)> {
        let mut buf = [0; 20];
        loop {
            match self.receiver.try_read(&mut buf) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(err) => return Err(anyhow::anyhow!(err)),
            }
        }
        Ok(self.handle.recv())
    }
}
