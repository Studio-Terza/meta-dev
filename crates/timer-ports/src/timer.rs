use port_core::port;

use crate::Moment;

#[port]
pub trait Timer {
    fn now(&self) -> Moment;
}
