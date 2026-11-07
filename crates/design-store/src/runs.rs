mod boundary;

use design_model::{JobId, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Started, Usage};
use named_derive::Named;
use port_core::Op;
use strum::VariantArray;
use transcript_ports::Body;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    #[name("running")]
    Running,
    #[name("replied")]
    Replied,
    #[name("overdue")]
    Overdue,
    #[name("killed")]
    Killed,
    #[name("gone")]
    Gone,
}

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    #[name("open")]
    Open,
    #[name("applied")]
    Applied,
    #[name("repeated")]
    Repeated,
    #[name("flailed")]
    Flailed,
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
    segments: Vec<Segment>,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct JobClosed {
    job: JobId,
    state: JobState,
}

#[derive(Clone, Debug, Op)]
pub struct RunSpawned {
    run: RunId,
    job: JobId,
    spawned: Spawned,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct RunEnded {
    run: RunId,
    state: RunState,
    usage: Option<Usage>,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct NextJob;

#[derive(Clone, Copy, Debug, Op)]
pub struct NextRun;

#[derive(Clone, Copy, Debug, Op)]
pub struct LiveRuns;

#[derive(Clone, Copy, Debug, Op)]
pub struct Unstructured;

#[derive(Clone, Copy, Debug, Op)]
pub struct ContextNodes;

#[derive(Clone, Copy, Debug, Op)]
pub struct RunsOf(JobId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ran {
    run: RunId,
    state: RunState,
    usage: Option<Usage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRun {
    run: RunId,
    spawned: Spawned,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unplaced {
    segment: Segment,
    body: Body,
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

impl JobOpened {
    #[must_use]
    pub fn new(job: JobId, segments: Vec<Segment>) -> JobOpened {
        JobOpened { job, segments }
    }
}

impl JobClosed {
    #[must_use]
    pub fn new(job: JobId, state: JobState) -> JobClosed {
        JobClosed { job, state }
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
        RunEnded { run, state, usage }
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
}

impl StoredRun {
    #[must_use]
    pub fn identity(&self) -> RunId {
        self.run
    }

    #[must_use]
    pub fn spawned(&self) -> &Spawned {
        &self.spawned
    }
}

impl AsRef<Segment> for Unplaced {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}

impl AsRef<Body> for Unplaced {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}
