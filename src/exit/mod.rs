mod handler;

use crate::timer::TimerBased;
pub(crate) use handler::ExitHandler;
use std::ops::ControlFlow;

pub(crate) struct ExitActor {
    handler: ExitHandler,
}

impl ExitActor {
    pub(crate) fn new() -> Self {
        let handler = ExitHandler::new();
        Self { handler }
    }

    pub(crate) fn handler(&self) -> ExitHandler {
        self.handler.clone()
    }
}

impl TimerBased for ExitActor {
    fn work(&mut self) -> ControlFlow<()> {
        if self.handler.received() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
