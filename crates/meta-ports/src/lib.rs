pub mod memory;
pub mod real;

use meta_core::{Cleared, Origin, PrNumber, Pull, Record, Timestamp};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError(String);

impl PortError {
    pub fn new(why: impl Into<String>) -> Self {
        Self(why.into())
    }
}

impl fmt::Display for PortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub trait Log {
    fn read(&self) -> Result<Vec<Record>, PortError>;
    fn append(&mut self, records: &[Record]) -> Result<(), PortError>;
}

pub trait Git {
    fn origin(&self) -> Result<Origin, PortError>;
    fn rules(&self) -> Result<String, PortError>;
}

pub trait GitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError>;
    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError>;
}

pub trait Clock {
    fn now(&self) -> Timestamp;
}

pub struct Ports<L, G, H, C> {
    pub log: L,
    pub git: G,
    pub github: H,
    pub clock: C,
}
