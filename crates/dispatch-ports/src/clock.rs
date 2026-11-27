use port_core::port;

use crate::{Pause, Wall};

#[port]
pub trait Clock {
    fn now(&self) -> Wall;

    fn paused(&self, pause: Pause) -> Wall;
}
