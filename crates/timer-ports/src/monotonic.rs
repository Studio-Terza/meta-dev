use std::sync::OnceLock;
use std::time::Instant;

use port_core::Pass;

use crate::Moment;
use crate::timer::{TimerFiller, timer_ops};

#[derive(Debug, Default)]
pub struct Monotonic {
    origin: OnceLock<Instant>,
}

impl TimerFiller for Monotonic {
    fn now(&self, _pass: Pass<'_, timer_ops::Now>) -> Moment {
        Moment::from(self.origin.get_or_init(Instant::now).elapsed())
    }
}
