use crate::dispatch::{Bell, Moment, Pause, Wall};
use crate::port::port;

#[port]
pub trait Clock {
    fn now(&self) -> Wall;

    fn moment(&self) -> Moment;

    fn waited(&self, pause: Pause, bell: &Bell) -> Wall;
}
