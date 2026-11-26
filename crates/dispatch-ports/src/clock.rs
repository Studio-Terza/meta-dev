use port_core::port;

use crate::{Bell, Moment, Pause, Wall};

#[port]
pub trait Clock {
    fn now(&self) -> Wall;

    fn moment(&self) -> Moment;

    fn waited(&self, pause: Pause, bell: &Bell) -> Wall;
}
