use crate::port;
use crate::timer::Moment;

#[port]
pub trait Timer {
    fn now(&self) -> Moment;
}
