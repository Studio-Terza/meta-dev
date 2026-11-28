use std::time::Duration;

use syntax::Clock;

use crate::{Moment, Timer};

#[derive(Debug)]
pub struct Timed<'timer, T> {
    timer: &'timer T,
    origin: Moment,
}

impl<'timer, T: Timer> Timed<'timer, T> {
    pub fn new(timer: &'timer T) -> Timed<'timer, T> {
        Timed { timer, origin: timer.now() }
    }
}

impl<T: Timer> Clock for Timed<'_, T> {
    fn now(&self) -> Duration {
        self.timer.now().since(self.origin).into()
    }
}
