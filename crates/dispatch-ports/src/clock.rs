use port_core::port;

use crate::{Bell, Pause, Wall};

#[port]
pub trait Clock {
    fn now(&self) -> Wall;

    fn waited(&self, pause: Pause, bell: &Bell) -> Wall;
}
