pub(crate) struct Timer {
    tick: u64,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self { tick: 0 }
    }

    pub(crate) fn tick(&mut self) {
        self.tick += 1;
    }

    pub(crate) fn passed(&mut self, ticks: u64) -> bool {
        if self.tick == ticks {
            self.tick = 0;
            true
        } else {
            false
        }
    }
}
