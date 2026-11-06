use std::fmt;
use std::io::Write;
use std::iter;

use design_model::{JobId, Provenance, RunId, Segment};
use design_store::{
    AppliedFile, ContextNodes, JobClosed, JobOpened, JobState, LiveRuns, NextJob, NextRun, RunEnded, RunSpawned,
    RunState, SpentOn, StoredRun, Unplaced, Unstructured,
};
use dispatch_ports::{
    AgentRun, Clock, Deadline, Dispatch, Due, Ended, Finished, Pause, Reply, Scratch, Spawned, Timeout, Usage,
};
use file_ports::{Bytes, Files};
use store_ports::Store;
use structurer::{Attempt, Directives, Job, Lapse, Reading, Said, SegmentRef};
use transcript_ports::Body;

use super::Printed;
use super::apply::{Application, Parsed};
use crate::cli::failure::Failure;
use crate::cli::request::{Crew, Ground};
use crate::{Jobs, Outcome};

#[derive(Debug)]
pub struct Structure<'run, S: Store, D: Dispatch, C: Clock> {
    store: &'run S,
    dispatch: &'run D,
    clock: &'run C,
    scratch: Scratch,
}

#[derive(Debug)]
struct Hands<'run, S: Store, D: Dispatch, C: Clock> {
    store: &'run S,
    dispatch: &'run D,
    clock: &'run C,
    scratch: &'run Scratch,
}

#[derive(Debug)]
struct Reaper<'hands, 'run, S: Store, D: Dispatch, C: Clock>(&'hands Hands<'run, S, D, C>);

#[derive(Debug)]
struct Jobber<'hands, 'run, S: Store, D: Dispatch, C: Clock>(&'hands Hands<'run, S, D, C>);

#[derive(Debug)]
struct Attempts<'hands, 'run, S: Store, D: Dispatch, C: Clock>(&'hands Hands<'run, S, D, C>);

#[derive(Debug)]
struct Watch<'hands, 'run, S: Store, D: Dispatch, C: Clock>(&'hands Hands<'run, S, D, C>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaping {
    Killed(RunId, Ended),
    Vanished(RunId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempted {
    run: RunId,
    state: RunState,
    usage: Option<Usage>,
}

#[derive(Debug)]
pub enum Ending {
    Applied(Application),
    Flailed(Lapse),
}

#[derive(Debug)]
pub struct Worked {
    job: JobId,
    runs: Vec<Attempted>,
    ending: Ending,
    spent: Usage,
}

#[derive(Debug)]
pub struct Structured {
    reaped: Vec<Reaping>,
    jobs: Vec<Worked>,
}

#[derive(Debug)]
enum Watched {
    Finished(Finished),
    Overdue,
}

#[derive(Debug)]
enum Step {
    Ended(Ending),
    Again(Attempt),
}

#[derive(Debug)]
enum Progress {
    Going(Vec<Worked>),
    Halted(Vec<Worked>),
}

impl<'run, S: Store, D: Dispatch, C: Clock> Structure<'run, S, D, C> {
    const JOB_SIZE: usize = 8;

    pub fn new<F: Files>(ground: &Ground<'run, F, S>, crew: &Crew<'run, D, C>) -> Structure<'run, S, D, C> {
        Structure {
            store: ground.store(),
            dispatch: crew.dispatch(),
            clock: crew.clock(),
            scratch: ground.located().scratch(),
        }
    }

    pub fn worked(&self) -> Result<Structured, Failure> {
        let hands = Hands { store: self.store, dispatch: self.dispatch, clock: self.clock, scratch: &self.scratch };
        let lock = self.store.lock()?;
        let reaped = Reaper(&hands).reaped()?;
        let unplaced = self.store.read(Unstructured)?;
        let progress = unplaced
            .chunks(Self::JOB_SIZE)
            .try_fold(Progress::Going(Vec::new()), |progress, chunk| progress.next(&Jobber(&hands), chunk))?;
        lock.end()?;
        Ok(Structured { reaped, jobs: progress.worked() })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Reaper<'_, '_, S, D, C> {
    fn reaped(&self) -> Result<Vec<Reaping>, Failure> {
        let now = self.0.clock.now();
        self.0
            .store
            .read(LiveRuns)?
            .into_iter()
            .filter(|stored| stored.spawned().deadline().due(now) == Due::Passed)
            .map(|stored| self.reap(&stored))
            .collect()
    }

    fn reap(&self, stored: &StoredRun) -> Result<Reaping, Failure> {
        let spawned = stored.spawned();
        let current = self.0.dispatch.started(spawned.pid())?;
        let same = spawned.started().filter(|recorded| current.as_ref() == Some(*recorded));
        let ended = same.map(|_same| self.0.dispatch.kill(spawned.pid())).transpose()?;
        let reaping =
            ended.map_or(Reaping::Vanished(stored.identity()), |ended| Reaping::Killed(stored.identity(), ended));
        self.0.store.write(RunEnded::new(stored.identity(), reaping.state(), None))?;
        Ok(reaping)
    }
}

impl<S: Store, D: Dispatch, C: Clock> Jobber<'_, '_, S, D, C> {
    fn job(&self, chunk: &[Unplaced]) -> Result<Worked, Failure> {
        let store = self.0.store;
        let id = store.read(NextJob)?;
        let context = store.read(ContextNodes)?;
        let said = chunk.iter().map(|unplaced| {
            let segment: &Segment = unplaced.as_ref();
            let body: &Body = unplaced.as_ref();
            Said::new(SegmentRef::from(segment), body.clone())
        });
        let job = Job::new(id, said.collect(), context);
        let segments = chunk.iter().map(|unplaced| AsRef::<Segment>::as_ref(unplaced).clone());
        store.write(JobOpened::new(id, segments.collect()))?;
        let worked = Attempts(self.0).attempted(&job, Attempt::First, Vec::new())?;
        store.write(JobClosed::new(id, worked.ending.state()))?;
        Ok(Worked { spent: store.read(SpentOn::from(id))?, ..worked })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Attempts<'_, '_, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(180);

    fn attempted(&self, job: &Job, attempt: Attempt, runs: Vec<Attempted>) -> Result<Worked, Failure> {
        let Hands { store, dispatch, clock, scratch } = self.0;
        let run = store.read(NextRun)?;
        let deadline = Deadline::counted_from(clock.now(), Self::TIMEOUT);
        let mut held = dispatch.dispatch(&job.prompt(&attempt), deadline, scratch)?;
        let spawned = Spawned::new(held.pid(), dispatch.started(held.pid())?, deadline);
        store.write(RunSpawned::new(run, job.identity(), spawned))?;
        let watched = Watch(self.0).watched(&mut held)?;
        held.end()?;
        let attempted = Attempted::ended(run, &watched);
        store.write(RunEnded::new(run, attempted.state, attempted.usage))?;
        let runs = runs.into_iter().chain([attempted]).collect();
        self.stepped(run, attempt, watched)?.continued(self, job, runs)
    }

    fn stepped(&self, run: RunId, attempt: Attempt, watched: Watched) -> Result<Step, Failure> {
        match watched {
            Watched::Overdue => Ok(Step::from(attempt.lapsed(Lapse::Overdue))),
            Watched::Finished(finished) => self.read(run, attempt, finished.reply()),
        }
    }

    fn read(&self, run: RunId, attempt: Attempt, reply: &Reply) -> Result<Step, Failure> {
        match attempt.read(reply) {
            Reading::Parsed { directives, attempt } => self.applied(run, attempt, directives, reply),
            reading @ (Reading::Again(_) | Reading::Flailed(_)) => Ok(Step::from(reading)),
        }
    }

    fn applied(&self, run: RunId, attempt: Attempt, directives: Directives, reply: &Reply) -> Result<Step, Failure> {
        let store = self.0.store;
        let parsed = Parsed::new(store, directives, Provenance::Agent(run));
        let applied = store.read(AppliedFile::from(Bytes::from(reply)))?.seen(parsed);
        applied
            .map(|application| Step::Ended(Ending::Applied(application)))
            .or_else(|failure| failure.refusal().map(|lapse| Step::from(attempt.lapsed(lapse))))
    }
}

impl<S: Store, D: Dispatch, C: Clock> Watch<'_, '_, S, D, C> {
    const PAUSE: Pause = Pause::new(2);

    fn watched(&self, run: &mut AgentRun) -> Result<Watched, Failure> {
        let deadline = run.deadline();
        iter::repeat_with(|| self.checked(deadline, run)).find_map(Result::transpose).unwrap_or(Ok(Watched::Overdue))
    }

    fn checked(&self, deadline: Deadline, run: &mut AgentRun) -> Result<Option<Watched>, Failure> {
        match deadline.due(self.0.clock.now()) {
            Due::Passed => Ok(Some(Watched::Overdue)),
            Due::Pending => self.polled(run).map(|output| output.map(Watched::Finished)),
        }
    }

    fn polled(&self, run: &mut AgentRun) -> Result<Option<Finished>, Failure> {
        let output = self.0.dispatch.output(run)?;
        let _woke = output.as_ref().map_or_else(|| Some(self.0.clock.waited(Self::PAUSE)), |_finished| None);
        Ok(output)
    }
}

impl Step {
    fn continued<S: Store, D: Dispatch, C: Clock>(
        self,
        attempts: &Attempts<'_, '_, S, D, C>,
        job: &Job,
        runs: Vec<Attempted>,
    ) -> Result<Worked, Failure> {
        match self {
            Step::Ended(ending) => Ok(Worked { job: job.identity(), runs, ending, spent: Usage::default() }),
            Step::Again(attempt) => attempts.attempted(job, attempt, runs),
        }
    }
}

impl From<Reading> for Step {
    fn from(reading: Reading) -> Step {
        match reading {
            Reading::Parsed { attempt, .. } | Reading::Again(attempt) => Step::Again(attempt),
            Reading::Flailed(lapse) => Step::Ended(Ending::Flailed(lapse)),
        }
    }
}

impl Progress {
    fn next<S: Store, D: Dispatch, C: Clock>(
        self,
        jobber: &Jobber<'_, '_, S, D, C>,
        chunk: &[Unplaced],
    ) -> Result<Progress, Failure> {
        match self {
            Progress::Halted(worked) => Ok(Progress::Halted(worked)),
            Progress::Going(worked) => Progress::extended(worked, jobber, chunk),
        }
    }

    fn extended<S: Store, D: Dispatch, C: Clock>(
        worked: Vec<Worked>,
        jobber: &Jobber<'_, '_, S, D, C>,
        chunk: &[Unplaced],
    ) -> Result<Progress, Failure> {
        let job = jobber.job(chunk)?;
        let next = job.ending.next();
        Ok(next(worked.into_iter().chain([job]).collect()))
    }

    fn worked(self) -> Vec<Worked> {
        match self {
            Progress::Going(worked) | Progress::Halted(worked) => worked,
        }
    }
}

impl Reaping {
    fn state(self) -> RunState {
        match self {
            Reaping::Killed(..) => RunState::Reaped,
            Reaping::Vanished(_) => RunState::Vanished,
        }
    }
}

impl Attempted {
    fn ended(run: RunId, watched: &Watched) -> Attempted {
        match watched {
            Watched::Finished(finished) => Attempted { run, state: RunState::Finished, usage: Some(finished.usage()) },
            Watched::Overdue => Attempted { run, state: RunState::Overdue, usage: None },
        }
    }
}

impl Ending {
    fn state(&self) -> JobState {
        match self {
            Ending::Applied(application) => application.state(),
            Ending::Flailed(_) => JobState::Flailed,
        }
    }

    fn next(&self) -> fn(Vec<Worked>) -> Progress {
        match self {
            Ending::Applied(_) => Progress::Going,
            Ending::Flailed(_) => Progress::Halted,
        }
    }
}

impl Worked {
    fn outcome(self, jobs: Jobs) -> Result<Outcome, Failure> {
        match self.ending {
            Ending::Applied(_) => Ok(Outcome::Structured(jobs)),
            Ending::Flailed(lapse) => Err(Failure::Flailed { job: self.job, lapse }),
        }
    }
}

impl Structured {
    pub(crate) fn reported(self, out: &mut impl Write) -> Result<Outcome, Failure> {
        let _printed = Printed::shown(out, &self)?;
        let jobs = Jobs::new(self.jobs.len());
        self.jobs.into_iter().last().map_or(Ok(Outcome::Structured(jobs)), |last| last.outcome(jobs))
    }
}

impl fmt::Display for Reaping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reaping::Killed(run, ended) => {
                writeln!(formatter, "reaped run {run} past its deadline: {ended}")
            }
            Reaping::Vanished(run) => {
                writeln!(formatter, "reaped run {run} past its deadline: its process is gone")
            }
        }
    }
}

impl fmt::Display for Attempted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "run {} {}", self.run, self.state)?;
        self.usage.iter().try_for_each(|usage| write!(formatter, " {usage}"))?;
        writeln!(formatter)
    }
}

impl fmt::Display for Ending {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ending::Applied(application) => write!(formatter, "applied\n{application}"),
            Ending::Flailed(lapse) => writeln!(formatter, "FLAILED: {lapse}"),
        }
    }
}

impl fmt::Display for Worked {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let job = self.job;
        self.runs.iter().try_for_each(|run| write!(formatter, "job {job} {run}"))?;
        writeln!(formatter, "job {job} {}", self.spent)?;
        write!(formatter, "job {job} {}", self.ending)
    }
}

impl fmt::Display for Structured {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reaped.iter().try_for_each(|reaping| reaping.fmt(formatter))?;
        self.jobs.iter().try_for_each(|worked| worked.fmt(formatter))?;
        writeln!(formatter, "structured {} jobs", self.jobs.len())
    }
}
