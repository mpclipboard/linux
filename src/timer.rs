use anyhow::{Context as _, Result};
use std::{collections::HashMap, ops::ControlFlow, thread::sleep, time::Duration};

type TimerFn = Box<dyn FnMut() -> ControlFlow<()>>;

pub(crate) struct Timer {
    ticks_count: u64,
    leap: u64,
    quantum: Duration,
    schedule: HashMap<u64, Vec<TimerFn>>,
}

impl Timer {
    pub(crate) fn new(quantum: Duration) -> Self {
        Self {
            ticks_count: 0,
            leap: 0,
            quantum,
            schedule: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, every: u64, f: impl (FnMut() -> ControlFlow<()>) + 'static) {
        self.schedule.entry(every).or_default().push(Box::new(f));
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        self.leap = lcm(self.schedule.keys().copied())?;

        loop {
            if self.tick().is_break() {
                return Ok(());
            }
        }
    }

    fn tick(&mut self) -> ControlFlow<()> {
        for (tick, fs) in self.schedule.iter_mut() {
            if self.ticks_count % *tick == 0 {
                for f in fs.iter_mut() {
                    (f)()?;
                }
            }
        }

        self.ticks_count += 1;

        if self.ticks_count == self.leap {
            self.ticks_count = 0;
        }

        sleep(self.quantum);

        ControlFlow::Continue(())
    }
}

fn gcd2(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd2(b, a % b) }
}

fn lcm2(a: u64, b: u64) -> u64 {
    a / gcd2(a, b) * b
}

fn lcm(nums: impl Iterator<Item = u64>) -> Result<u64> {
    nums.reduce(lcm2)
        .context("expected at least two elements to compute LCM")
}
