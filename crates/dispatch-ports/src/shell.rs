mod boundary;

use port_core::port;

use crate::DispatchFailure;
pub use boundary::{Call, Printed};

#[port]
pub trait Shell {
    fn ran(&self, call: &Call) -> Result<Ran, DispatchFailure>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ran {
    Succeeded(Printed),
    Failed(Printed),
    Overdue,
}

impl Ran {
    #[must_use]
    pub fn printed(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) | Ran::Failed(printed) => Some(printed),
            Ran::Overdue => None,
        }
    }

    #[must_use]
    pub fn succeeded(&self) -> Option<&Printed> {
        match self {
            Ran::Succeeded(printed) => Some(printed),
            Ran::Failed(_) | Ran::Overdue => None,
        }
    }
}
