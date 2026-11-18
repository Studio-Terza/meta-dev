mod boundary;
mod lifecycles;

pub use boundary::{Release, ReleaseNew, Succession, SuccessionNew};
pub use lifecycles::{Delivery, RunState, delivery, run};

use std::fmt;

use design_jobs::JobState;
use design_model::{JobId, RunId, Segment};
use dispatch_ports::{Deadline, Finished, Pid, Reply, Stall, Stalled, Started, Usage};
use named_derive::Named;
use newtype::{Arbitrary, Display, Newtype, Wrapping};
use port_core::Op;
use transcript_ports::{Body, SessionId};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Account(String);

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Excerpt(String);

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halted {
    #[name("flailed")]
    Flailed,
    #[name("stalled")]
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawned {
    pid: Pid,
    started: Option<Started>,
    deadline: Deadline,
}

#[derive(Clone, Debug, Op)]
pub struct JobOpened {
    job: JobId,
    covering: Vec<Segment>,
}

#[derive(Clone, Debug, Op)]
pub struct JobHalted {
    job: JobId,
    halted: Halted,
    account: Account,
    held: Option<JobFlailed>,
    retold: Vec<Retold>,
}

#[derive(Clone, Debug, Op)]
pub struct Retold {
    job: JobId,
    account: Account,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct UnreportedFailures;

#[derive(Debug, Op)]
pub struct FailuresReported(Vec<delivery::Unreported>);

#[derive(Clone, Copy, Debug, Op)]
pub struct JobsInterrupted;

#[derive(Clone, Debug, Op)]
pub struct RunSpawned {
    run: RunId,
    job: JobId,
    spawned: Spawned,
}

#[derive(Clone, Debug, Op)]
pub struct RunEnded {
    run: RunId,
    state: RunState,
    usage: Option<Usage>,
    reply: Option<Excerpt>,
}

#[derive(Clone, Debug)]
pub struct JobFlailed {
    run: RunId,
    segments: Vec<Segment>,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct JobRetried(JobId);

#[derive(Clone, Copy, Debug, Op)]
pub struct NextJob;

#[derive(Clone, Copy, Debug, Op)]
pub struct NextRun;

#[derive(Clone, Copy, Debug, Op)]
pub struct LiveRuns;

#[derive(Clone, Copy, Debug, Op)]
pub struct Unstructured;

#[derive(Clone, Copy, Debug, Op)]
pub struct UnplacedSegments;

#[derive(Clone, Copy, Debug, Op)]
pub struct RunsOf(JobId);

#[derive(Clone, Copy, Debug, Op)]
pub struct FlailedSegments;

#[derive(Clone, Copy, Debug, Op)]
pub struct HeldBy(JobId);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    run: RunId,
    state: RunState,
    usage: Option<Usage>,
    reply: Option<Excerpt>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredRun {
    held: run::Running,
    spawned: Spawned,
}

#[derive(Debug, Op)]
pub struct RunReaped {
    held: run::Running,
    ended: run::Ended,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flailed {
    segment: Segment,
    job: JobId,
    run: RunId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hold {
    first: Flailed,
    rest: Vec<Flailed>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unreported {
    held: delivery::Unreported,
    account: Account,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unplaced {
    segment: Segment,
    body: Body,
    session: SessionId,
    flailed: Option<RunId>,
}

impl Spawned {
    #[must_use]
    pub fn new(pid: Pid, started: Option<Started>, deadline: Deadline) -> Spawned {
        Spawned { pid, started, deadline }
    }

    #[must_use]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    #[must_use]
    pub fn started(&self) -> Option<&Started> {
        self.started.as_ref()
    }
}

impl AsRef<Deadline> for Spawned {
    fn as_ref(&self) -> &Deadline {
        &self.deadline
    }
}

impl From<&Stall> for RunState {
    fn from(stall: &Stall) -> RunState {
        stall.stalled()
    }
}

impl Stalled for RunState {
    const ERRORED: RunState = RunState::Errored;
    const UNREADABLE: RunState = RunState::Unreadable;
    const OVERDUE: RunState = RunState::Overdue;
    const UNSPAWNED: RunState = RunState::Failed;
    const FAILED: RunState = RunState::Failed;
}

impl From<JobId> for JobOpened {
    fn from(job: JobId) -> JobOpened {
        JobOpened { job, covering: Vec::new() }
    }
}

impl JobOpened {
    #[must_use]
    pub fn covering(self, covering: Vec<Segment>) -> JobOpened {
        JobOpened { covering, ..self }
    }
}

impl JobHalted {
    #[must_use]
    pub fn new(job: JobId, halted: Halted, account: Account) -> JobHalted {
        JobHalted { job, halted, account, held: None, retold: Vec::new() }
    }

    #[must_use]
    pub fn holding(self, held: Option<JobFlailed>) -> JobHalted {
        JobHalted { held, ..self }
    }

    #[must_use]
    pub fn retelling(self, retold: Vec<Retold>) -> JobHalted {
        JobHalted { retold, ..self }
    }
}

impl Retold {
    #[must_use]
    pub fn new(job: JobId, account: Account) -> Retold {
        Retold { job, account }
    }
}

impl From<Halted> for JobState {
    fn from(halted: Halted) -> JobState {
        match halted {
            Halted::Flailed => JobState::Flailed,
            Halted::Stalled => JobState::Stalled,
        }
    }
}

impl FromIterator<Unreported> for FailuresReported {
    fn from_iter<Failures: IntoIterator<Item = Unreported>>(failures: Failures) -> FailuresReported {
        FailuresReported(failures.into_iter().map(|failure| failure.held).collect())
    }
}

impl AsRef<Account> for Unreported {
    fn as_ref(&self) -> &Account {
        &self.account
    }
}

impl RunSpawned {
    #[must_use]
    pub fn new(run: RunId, job: JobId, spawned: Spawned) -> RunSpawned {
        RunSpawned { run, job, spawned }
    }
}

impl RunEnded {
    #[must_use]
    pub fn new(run: RunId, state: RunState, usage: Option<Usage>) -> RunEnded {
        RunEnded { run, state, usage, reply: None }
    }

    #[must_use]
    pub fn finished(run: RunId, finished: &Finished) -> RunEnded {
        RunEnded {
            run,
            state: RunState::Replied,
            usage: Some(finished.usage()),
            reply: Some(Excerpt::from(finished.reply())),
        }
    }
}

impl JobFlailed {
    #[must_use]
    pub fn new(run: RunId, segments: Vec<Segment>) -> JobFlailed {
        JobFlailed { run, segments }
    }
}

impl From<&Hold> for JobRetried {
    fn from(hold: &Hold) -> JobRetried {
        JobRetried(hold.first.job)
    }
}

impl From<JobId> for HeldBy {
    fn from(job: JobId) -> HeldBy {
        HeldBy(job)
    }
}

impl Hold {
    pub fn held(&self) -> impl Iterator<Item = &Flailed> {
        [&self.first].into_iter().chain(&self.rest)
    }
}

impl From<JobId> for RunsOf {
    fn from(job: JobId) -> RunsOf {
        RunsOf(job)
    }
}

impl Ran {
    #[must_use]
    pub fn identity(&self) -> RunId {
        self.run
    }

    #[must_use]
    pub fn state(&self) -> RunState {
        self.state
    }

    #[must_use]
    pub fn usage(&self) -> Option<Usage> {
        self.usage
    }

    #[must_use]
    pub fn reply(&self) -> Option<&Excerpt> {
        self.reply.as_ref()
    }
}

impl StoredRun {
    #[must_use]
    pub fn identity(&self) -> RunId {
        *self.held.key()
    }

    #[must_use]
    pub fn spawned(&self) -> &Spawned {
        &self.spawned
    }

    #[must_use]
    pub fn reaped(self, ended: run::Ended) -> RunReaped {
        RunReaped { held: self.held, ended }
    }
}

impl Flailed {
    #[must_use]
    pub fn job(&self) -> JobId {
        self.job
    }

    #[must_use]
    pub fn run(&self) -> RunId {
        self.run
    }
}

impl AsRef<Segment> for Flailed {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}

impl AsRef<Segment> for Unplaced {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}

impl AsRef<SessionId> for Unplaced {
    fn as_ref(&self) -> &SessionId {
        &self.session
    }
}

impl AsRef<Body> for Unplaced {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}

impl Account {
    #[must_use]
    pub fn told(told: &impl fmt::Display) -> Account {
        Account(told.to_string())
    }
}

impl Excerpt {
    pub const BYTES: usize = 16 * 1024;
}

impl From<&Reply> for Excerpt {
    fn from(reply: &Reply) -> Excerpt {
        let told = reply.clone().into_inner();
        let kept = told.get(..told.floor_char_boundary(Excerpt::BYTES)).unwrap_or_default();
        Excerpt(kept.to_owned())
    }
}
