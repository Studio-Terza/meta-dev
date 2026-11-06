use port_core::{Calls, OpName, RootRefused};

use crate::Scale;

#[derive(Debug)]
pub enum Failure {
    Refused(RootRefused),
    Ran(Box<dyn std::fmt::Debug>),
    Exceeded { scale: Scale, name: OpName, limit: Calls, calls: Calls },
    Superlinear { name: OpName, base: Calls, scaled: Calls, factor: Scale },
}

impl From<RootRefused> for Failure {
    fn from(refused: RootRefused) -> Failure {
        Failure::Refused(refused)
    }
}
