use std::time::{Duration, Instant};

pub struct CallEvery<const PERIOD_MS: u64> {
    next_call: Instant,
}

impl<const PERIOD_MS: u64> CallEvery<PERIOD_MS> {
    fn period() -> Duration {
        Duration::from_millis(PERIOD_MS)
    }

    pub fn new() -> Self {
        CallEvery {
            next_call: Instant::now() + Self::period(),
        }
    }

    pub fn call(&mut self, f: impl FnOnce()) {
        // if call is due
        if Instant::now() >= self.next_call {
            // schedule next call in a period
            self.next_call += Self::period();
            // call closure
            f()
        }
    }
}
