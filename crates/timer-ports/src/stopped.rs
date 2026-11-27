use port_core::Pass;

use crate::Moment;
use crate::timer::{TimerFiller, timer_ops};

#[derive(Clone, Copy, Debug, Default)]
pub struct Stopped(Moment);

impl TimerFiller for Stopped {
    fn now(&self, _pass: Pass<'_, timer_ops::Now>) -> Moment {
        self.0
    }
}
