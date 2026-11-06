use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use port_core::Pass;

use crate::{ClockFiller, Pause, Wall, clock_ops};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl ClockFiller for SystemClock {
    fn now(&self, _pass: Pass<'_, clock_ops::Now>) -> Wall {
        Wall::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default())
    }

    fn waited(&self, _pass: Pass<'_, clock_ops::Waited>, pause: Pause) -> Wall {
        thread::sleep(Duration::from(pause));
        Wall::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default())
    }
}
