use crate::{Clock, GhReplies, Git, GitHub, Log, PortError};
use meta_core::{Cleared, Origin, PrNumber, Pull, Record, SelfFile, Sha, Timestamp};
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

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        self.open()?;
        let (records, decided) = decide(Ok(self.records.clone()));
        self.records.extend(records);
        Ok(decided)
    }
}

#[derive(Debug)]
pub struct RecordedGit {
    pub origin: Origin,
    /// The files on main.
    pub files: BTreeMap<SelfFile, String>,
    /// The files in the working tree or the cache.
    pub local: BTreeMap<SelfFile, String>,
}

impl Git for RecordedGit {
    fn origin(&self) -> Result<Origin, PortError> {
        Ok(self.origin.clone())
    }

    fn read(&self, file: SelfFile) -> Result<String, PortError> {
        self.files.get(&file).cloned().ok_or_else(|| PortError::new(format!("no recorded {}", file.path())))
    }

    fn local(&self, file: SelfFile) -> Result<String, PortError> {
        let missing = || PortError::new(format!("no local {} in the working tree or the cache", file.path()));
        self.local.get(&file).cloned().ok_or_else(missing)
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
