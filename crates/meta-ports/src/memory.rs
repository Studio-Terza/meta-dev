use crate::{Clock, GhReplies, Git, GitHub, Log, PortError, Sessions};
use meta_core::{
    AgentId, Binary, Cleared, Head, Marking, Origin, Place, PrNumber, ProcessGroup, Pull, Record, SelfFile, Session,
    Sha, SideLine, Span, Standing, TaskId, Timestamp, UnverifiedEscape, UnverifiedEscapes, Updating, WorkerPlan,
    WorkerRun,
};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Default)]
pub struct MemoryLog {
    pub records: Vec<Record>,
    pub broken: Option<PortError>,
    /// Where the log stands: the default data dir, or anywhere else.
    pub place: Place,
    /// The side file beside the log: its lines, `None` for one that cannot be read.
    pub set_aside: Vec<Option<UnverifiedEscape>>,
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

    fn standing(&self) -> Standing {
        Head::of(self.open().map(|()| self.records.first())).standing(self.place)
    }

    fn mark(&mut self, marker: Record) -> Result<Marking, PortError> {
        self.open()?;
        match Head::of::<PortError>(Ok(self.records.first())) {
            Head::Marked => Ok(Marking::AlreadyMarked),
            _ => {
                self.records.insert(0, marker);
                Ok(Marking::Added)
            }
        }
    }

    fn read(&self) -> Result<Vec<Record>, PortError> {
        self.open().map(|()| self.records.clone())
    }

    fn set_aside(&mut self, escape: &UnverifiedEscape) -> Result<String, PortError> {
        self.set_aside.push(Some(escape.clone()));
        Ok("memory side file".to_owned())
    }

    fn set_aside_escapes(&self) -> Result<UnverifiedEscapes, PortError> {
        Ok(UnverifiedEscapes::of(self.set_aside.iter().cloned()))
    }

    fn import_set_aside<T>(
        &mut self,
        decide: impl FnOnce(
            Standing,
            Result<Vec<Record>, PortError>,
            Vec<SideLine>,
        ) -> (Vec<Record>, Option<Vec<SideLine>>, T),
    ) -> Result<T, PortError> {
        self.open()?;
        let standing = self.standing();
        let lines = self
            .set_aside
            .iter()
            .map(|line| match line {
                Some(escape) => SideLine::new(serde_json::to_vec(escape).unwrap_or_default(), Some(escape.clone())),
                None => SideLine::new(b"unreadable".to_vec(), None),
            })
            .collect();
        let (records, kept, decided) = decide(standing, Ok(self.records.clone()), lines);
        self.records.extend(records);
        if let Some(kept) = kept {
            self.set_aside = kept.iter().map(|line| line.escape().cloned()).collect();
        }
        Ok(decided)
    }

    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Standing, Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError> {
        self.open()?;
        let standing = self.standing();
        let (records, decided) = decide(standing, Ok(self.records.clone()));
        self.records.extend(records);
        Ok(decided)
    }
}

#[derive(Debug)]
pub struct RecordedGit {
    pub origin: Origin,
    /// The path of the running binary, as recorded.
    pub binary: Binary,
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

    fn binary(&self) -> Result<Binary, PortError> {
        Ok(self.binary.clone())
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
    pub updated: Vec<PrNumber>,
    pub listed: Option<Result<String, PortError>>,
    /// The reply to the one open-PR list call.
    pub open: Option<Result<String, PortError>>,
}

impl GitHub for RecordedGitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError> {
        let earlier = self.earlier.borrow_mut().get_mut(&pr).and_then(VecDeque::pop_front);
        earlier
            .or_else(|| self.replies.get(&pr).cloned())
            .ok_or_else(|| PortError::new(format!("no recorded reply for {pr}")))?
            .pull()
    }

    fn runs(&self, pr: PrNumber) -> Result<String, PortError> {
        self.replies
            .get(&pr)
            .map(|replies| replies.runs.clone())
            .ok_or_else(|| PortError::new(format!("no recorded reply for {pr}")))
    }

    fn merged_list(&self) -> Result<String, PortError> {
        self.listed.clone().unwrap_or_else(|| Err(PortError::new("no recorded merged list")))
    }

    fn open_list(&self) -> Result<String, PortError> {
        self.open.clone().unwrap_or_else(|| Err(PortError::new("no recorded open list")))
    }

    fn update_branch(&mut self, updating: &Updating) -> Result<(), PortError> {
        self.updated.push(updating.pr());
        Ok(())
    }

    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError> {
        self.merged.push((cleared.pr(), cleared.head().clone()));
        Ok(())
    }
}

/// Sessions that only record what they were asked; no process runs.
#[derive(Debug, Default)]
pub struct RecordedSessions {
    pub started: Vec<WorkerPlan>,
    pub stopped: Vec<WorkerRun>,
    /// Why starting fails, when it does.
    pub broken_start: Option<PortError>,
    /// Why stopping fails, when it does.
    pub broken_stop: Option<PortError>,
    /// The replies to `state`, one per call; after them a session is running.
    pub states: RefCell<VecDeque<Result<Session, PortError>>>,
}

impl RecordedSessions {
    /// The first process group handed out.
    pub const FIRST_GROUP: u32 = 900;

    pub fn transcript(worker: &AgentId) -> String {
        format!("/recorded/workers/{worker}/transcript.jsonl")
    }
}

impl Sessions for RecordedSessions {
    fn start(&mut self, plan: &WorkerPlan) -> Result<WorkerRun, PortError> {
        self.broken_start.clone().map_or(Ok(()), Err)?;
        self.started.push(plan.clone());
        let group = Self::FIRST_GROUP.saturating_add(u32::try_from(self.started.len()).unwrap_or(u32::MAX));
        let transcript = Self::transcript(plan.id()).parse()?;
        Ok(WorkerRun { group: ProcessGroup::of(group), transcript })
    }

    fn state(&self, _: &AgentId, _: &WorkerRun) -> Result<Session, PortError> {
        self.states.borrow_mut().pop_front().unwrap_or(Ok(Session::Running))
    }

    fn stop(&mut self, run: &WorkerRun) -> Result<(), PortError> {
        self.broken_stop.clone().map_or(Ok(()), Err)?;
        self.stopped.push(run.clone());
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

    fn pause(&self, span: Span) {
        self.0.set(self.0.get().plus(span));
    }
}
