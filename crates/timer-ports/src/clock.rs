use std::thread;

use port_core::{Pass, Real, port};

use crate::{Pause, Wall};

#[port]
pub trait Clock {
    fn now(&self) -> Wall;

    fn paused(&self, pause: Pause) -> Wall;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Real for SystemClock {}

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::now()
    }

    fn paused(&self, _pass: Pass<'_, clock_ops::Paused>, pause: Pause) -> Wall {
        thread::sleep(pause.into());
        Wall::now()
    }
}
