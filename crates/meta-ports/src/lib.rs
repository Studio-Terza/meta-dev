pub mod memory;
pub mod real;

use meta_core::{Cleared, Malformed, Origin, PrNumber, Pull, Record, Timestamp};
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

impl From<Malformed> for PortError {
    fn from(malformed: Malformed) -> Self {
        Self(malformed.to_string())
    }
}

pub trait Log {
    fn place(&self) -> String;
    fn read(&self) -> Result<Vec<Record>, PortError>;
    fn append(&mut self, records: &[Record]) -> Result<(), PortError>;
    fn transact<T>(&mut self, decide: impl FnOnce(Vec<Record>) -> (Vec<Record>, T)) -> Result<T, PortError>;
}

pub trait Git {
    fn origin(&self) -> Result<Origin, PortError>;
    fn rules(&self) -> Result<String, PortError>;
}

#[derive(Debug, Clone)]
pub struct GhReplies {
    pub view: String,
    pub required: Result<String, PortError>,
    pub runs: String,
}

impl GhReplies {
    pub fn pull(self) -> Result<Pull, PortError> {
        Ok(Pull::from_gh(&self.view, &self.required?, &self.runs)?)
    }
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
