use anyhow::{Context as _, Result};
use mpclipboard_generic_client::{
    Clip, Config, ConfigReadOption, Event, Handle, Logger, Store, TLS, Thread,
};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use tokio::net::unix::pipe::Receiver;

pub(crate) struct MPClipboard {
    handle: Handle,
    store: Store,
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

        Ok(Self {
            handle,
            store: Store::new(),
            receiver,
        })
    }

    pub(crate) fn stop(self) -> Result<()> {
        self.handle.stop()
    }

    pub(crate) fn send(&mut self, clip: Clip) -> Result<()> {
        if self.store.add(&clip) {
            self.handle.send(&clip.text)?;
        }
        Ok(())
    }

    fn poll(&mut self) -> Vec<Event> {
        let mut events = vec![];
        let (clip, connectivity) = self.handle.recv();
        if let Some(clip) = clip {
            events.push(Event::NewClip(clip));
        }
        if let Some(connectivity) = connectivity {
            events.push(Event::ConnectivityChanged(connectivity));
        }
        events
    }

    async fn wait_readable(&mut self) -> Result<()> {
        self.receiver.readable().await?;
        let mut buf = [0; 20];
        loop {
            match self.receiver.try_read(&mut buf) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(());
                }
                Err(err) => return Err(anyhow::anyhow!(err)),
            }
        }
    }

    pub(crate) async fn recv(&mut self) -> Result<Vec<Event>> {
        self.wait_readable().await?;
        Ok(self.poll())
    }
}
