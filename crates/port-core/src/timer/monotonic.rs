use std::sync::OnceLock;
use std::time::Instant;

use crate::Pass;
use crate::timer::Moment;
use crate::timer::timing::{TimerFiller, timer_ops};

#[derive(Debug, Default)]
pub struct Monotonic {
    origin: OnceLock<Instant>,
}

impl TimerFiller for Monotonic {
    fn now(&self, _pass: Pass<'_, timer_ops::Now>) -> Moment {
        Moment::from(self.origin.get_or_init(Instant::now).elapsed())
    }
}
