use crate::{Clock, GhReplies, Git, GitHub, Log, PortError};
use meta_core::{Cleared, Origin, PrNumber, Pull, Record, SelfFile, Sha, TaskId, Timestamp};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Default)]
pub struct MemoryLog {
    pub records: Vec<Record>,
    pub broken: Option<PortError>,
    /// Stands for the lead's real log, which only a main build may write.
    pub real: bool,
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

    fn is_real(&self) -> bool {
        self.real
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
    /// The tip of each local branch named after a task.
    pub branches: BTreeMap<TaskId, Sha>,
    /// Whether each commit is on origin/main; any other commit is a port failure.
    pub on_main: BTreeMap<Sha, bool>,
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

    fn local_head(&self, task: TaskId) -> Result<Option<Sha>, PortError> {
        Ok(self.branches.get(&task).cloned())
    }

    fn on_main(&self, commit: &Sha) -> Result<bool, PortError> {
        self.on_main.get(commit).copied().ok_or_else(|| PortError::new(format!("no recorded ancestry for {commit}")))
    }
}

#[derive(Debug, Default)]
pub struct RecordedGitHub {
    pub replies: BTreeMap<PrNumber, GhReplies>,
    /// Replies given first, one per call, before `replies` takes over: GitHub lagging behind a push.
    pub earlier: RefCell<BTreeMap<PrNumber, VecDeque<GhReplies>>>,
    pub merged: Vec<(PrNumber, Sha)>,
    pub listed: Option<Result<String, PortError>>,
}

impl GitHub for RecordedGitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        let earlier = self.earlier.borrow_mut().get_mut(&pr).and_then(VecDeque::pop_front);
        earlier
            .or_else(|| self.replies.get(&pr).cloned())
            .ok_or_else(|| PortError::new(format!("no recorded reply for {pr}")))?
            .pull()
    }

    fn merged_list(&self) -> Result<String, PortError> {
        self.listed.clone().unwrap_or_else(|| Err(PortError::new("no recorded merged list")))
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        self.merged.push((cleared.pr(), cleared.head().clone()));
        Ok(())
    }
}

/// A clock that stands still until it is told to wait, so that tests never sleep.
#[derive(Debug)]
pub struct FakeClock(Cell<Timestamp>);

impl FakeClock {
    pub fn at(start: Timestamp) -> Self {
        Self(Cell::new(start))
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        self.0.get()
    }

    fn pause(&self, seconds: u64) {
        self.0.set(self.0.get().advanced(seconds));
    }
}
