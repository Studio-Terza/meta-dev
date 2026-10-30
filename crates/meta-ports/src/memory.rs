use crate::{Clock, Git, GitHub, Log, PortError};
use meta_core::{Cleared, Origin, PrNumber, Pull, Record, Sha, Timestamp};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MemoryLog {
    pub records: Vec<Record>,
}

impl Log for MemoryLog {
    fn read(&self) -> Result<Vec<Record>, PortError> {
        Ok(self.records.clone())
    }

    fn append(&mut self, records: &[Record]) -> Result<(), PortError> {
        self.records.extend_from_slice(records);
        Ok(())
    }
}

#[derive(Debug)]
pub struct RecordedGit {
    pub origin: Origin,
    pub rules: String,
}

impl Git for RecordedGit {
    fn origin(&self) -> Result<Origin, PortError> {
        Ok(self.origin.clone())
    }

    fn rules(&self) -> Result<String, PortError> {
        Ok(self.rules.clone())
    }
}

#[derive(Debug, Default)]
pub struct RecordedGitHub {
    pub pulls: BTreeMap<PrNumber, Pull>,
    pub merged: Vec<(PrNumber, Sha)>,
}

impl GitHub for RecordedGitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        self.pulls.get(&pr).cloned().ok_or_else(|| PortError::new(format!("no recorded reply for {pr}")))
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        self.merged.push((cleared.pr(), cleared.head().clone()));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Timestamp);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.0
    }
}
