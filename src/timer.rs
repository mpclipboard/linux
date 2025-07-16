use anyhow::{Context as _, Result};
use std::{collections::HashMap, ops::ControlFlow, thread::sleep, time::Duration};

pub(crate) struct Timer {
    ticks_count: u64,
    leap: u64,
    quantum: Duration,
    schedule: HashMap<u64, Vec<Box<dyn TimerBased>>>,
}

pub(crate) trait TimerBased {
    fn work(&mut self) -> ControlFlow<()>;
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

    pub(crate) fn add(&mut self, every: Duration, f: impl TimerBased + 'static) {
        let every = (every.as_millis() / self.quantum.as_millis()) as u64;
        self.schedule.entry(every).or_default().push(Box::new(f));
    }

    pub(crate) fn start(&mut self) -> Result<()> {
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

        self.leap = lcm(self.schedule.keys().copied())?;

        loop {
            if self.tick().is_break() {
                return Ok(());
            }
        }
    }

    fn tick(&mut self) -> ControlFlow<()> {
        for (every, actors) in self.schedule.iter_mut() {
            if self.ticks_count % *every == 0 {
                for actor in actors.iter_mut() {
                    actor.work()?;
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
