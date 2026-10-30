use crate::{Clock, GhReplies, Git, GitHub, Log, PortError};
use meta_core::{Cleared, Origin, PrNumber, Pull, Record, Sha, Timestamp};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MemoryLog {
    pub records: Vec<Record>,
    pub broken: Option<PortError>,
}

impl MemoryLog {
    fn open(&self) -> Result<(), PortError> {
        self.broken.clone().map_or(Ok(()), Err)
    }
}

impl Log for MemoryLog {
    fn place(&self) -> String {
        "memory log".to_owned()
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        self.open().map(|()| self.records.clone())
    }

    fn append(&mut self, records: &[Record]) -> Result<(), PortError> {
        self.open()?;
        self.records.extend_from_slice(records);
        Ok(())
    }

    fn transact<T>(&mut self, decide: impl FnOnce(Vec<Record>) -> (Vec<Record>, T)) -> Result<T, PortError> {
        self.open()?;
        let (records, decided) = decide(self.records.clone());
        self.records.extend(records);
        Ok(decided)
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
    pub replies: BTreeMap<PrNumber, GhReplies>,
    pub merged: Vec<(PrNumber, Sha)>,
    pub listed: Option<Result<String, PortError>>,
}

impl GitHub for RecordedGitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        self.replies.get(&pr).cloned().ok_or_else(|| PortError::new(format!("no recorded reply for {pr}")))?.pull()
    }

    fn merged_list(&self) -> Result<String, PortError> {
        self.listed.clone().unwrap_or_else(|| Err(PortError::new("no recorded merged list")))
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
