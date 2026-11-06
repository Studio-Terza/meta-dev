use newtype::Counter;
use port_core::{Calls, OpName};

use crate::{Failure, Scale};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complexity {
    Linear,
}

impl Complexity {
    pub fn admits(self, name: OpName, base: Calls, scaled: Calls, factor: Scale) -> Result<(), Failure> {
        match self {
            Complexity::Linear => Some(scaled)
                .filter(|made| made.count() > base.count().saturating_mul(factor.count()))
                .map_or(Ok(()), |made| Err(Failure::Superlinear { name, base, scaled: made, factor })),
        }
    }
}
