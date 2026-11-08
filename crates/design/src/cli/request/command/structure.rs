use std::io::Write;

use design_model::{JobId, Provenance, RunId, Segment};
use design_store::{
    ContextNodes, JobOpened, LiveRuns, NextJob, NextRun, RunEnded, RunSpawned, RunState, RunsOf, Spawned, StoredRun,
    Unplaced, Unstructured,
};
use dispatch_ports::{Clock, Deadline, Dispatch, Due, Ended, Finished, Job, Prompt, Timeout, Watch, Watched};
use store_ports::Store;
use structurer::{Attempt, Lapse, Reading, Said, SegmentRef};
use transcript_ports::Body;

use super::Printed;
use super::apply::{Application, Parsed};
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::output::{Attempted, Count, Lapsed, Line};

#[derive(Debug)]
pub struct Crew<'ports, D: Dispatch, C: Clock> {
    dispatch: &'ports D,
    clock: &'ports C,
}

#[derive(Debug)]
pub struct Structure<'ports, S: Store, D: Dispatch, C: Clock> {
    store: &'ports S,
    crew: Crew<'ports, D, C>,
}

#[derive(Debug)]
struct Reaper<'structure, 'ports, S: Store, D: Dispatch, C: Clock>(&'structure Structure<'ports, S, D, C>);

#[derive(Debug)]
struct Attempts<'structure, 'ports, S: Store, D: Dispatch, C: Clock> {
    structure: &'structure Structure<'ports, S, D, C>,
    id: JobId,
    job: &'structure structurer::Job,
}

#[derive(Debug)]
enum Ending {
    Applied(Application),
    Flailed(Lapse),
}

#[derive(Debug)]
struct Concluded {
    retried: Option<Lapse>,
    ending: Ending,
}

#[derive(Debug)]
struct Worked {
    job: JobId,
    flailed: Option<Lapse>,
    lines: Vec<Line>,
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

#[derive(Clone, Copy, Debug)]
struct Reaped {
    run: RunId,
    state: RunState,
}

#[derive(Debug)]
pub struct Structured {
    reaped: Vec<Reaped>,
    worked: Vec<Worked>,
}

impl<'ports, D: Dispatch, C: Clock> Crew<'ports, D, C> {
    pub fn new(dispatch: &'ports D, clock: &'ports C) -> Crew<'ports, D, C> {
        Crew { dispatch, clock }
    }
}

impl<'ports, S: Store, D: Dispatch, C: Clock> Structure<'ports, S, D, C> {
    const TIMEOUT: Timeout = Timeout::new(180);

    pub fn new(store: &'ports S, crew: &Crew<'ports, D, C>) -> Structure<'ports, S, D, C> {
        Structure { store, crew: Crew { dispatch: crew.dispatch, clock: crew.clock } }
    }

    pub fn worked(&self) -> Result<Structured, Failure> {
        let lock = self.store.lock()?;
        let now = self.crew.clock.now();
        let reaped = self
            .store
            .read(LiveRuns)?
            .iter()
            .filter(|stored| AsRef::<Deadline>::as_ref(stored.spawned()).due(now) == Due::Passed)
            .map(|stored| Reaper(self).reaped(stored))
            .collect::<Result<Vec<Reaped>, Failure>>()?;
        let unplaced = self.store.read(Unstructured)?;
        let progress = unplaced
            .chunks(structurer::Job::SEGMENTS)
            .try_fold(Progress::Going(Vec::new()), |progress, chunk| progress.next(self, chunk))?;
        lock.end()?;
        Ok(Structured { reaped, worked: progress.worked() })
    }

    fn job(&self, chunk: &[Unplaced]) -> Result<Worked, Failure> {
        let id = self.store.read(NextJob)?;
        let context = self.store.read(ContextNodes)?;
        let said = chunk.iter().map(|unplaced| {
            Said::new(SegmentRef::from(AsRef::<Segment>::as_ref(unplaced)), AsRef::<Body>::as_ref(unplaced).clone())
        });
        let job = structurer::Job::new(said.collect(), &context);
        self.store.write(JobOpened::from(id))?;
        let ended = Attempts { structure: self, id, job: &job }.attempted(Attempt::First)?;
        let retried = ended.retried.iter().map(|lapse| Line::Retried(Lapsed::new(id, lapse.clone())));
        let closing = ended.ending.lines(id, self.store)?;
        let runs = self.store.read(RunsOf::from(id))?;
        let ran = runs.iter().map(|ran| Line::Ran(Attempted::new(ran.identity(), id, ran.state(), ran.usage())));
        let lines = retried.chain(closing).chain(ran).collect();
        Ok(Worked { job: id, flailed: ended.ending.flailed().cloned(), lines })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Reaper<'_, '_, S, D, C> {
    fn reaped(&self, stored: &StoredRun) -> Result<Reaped, Failure> {
        let Structure { store, crew } = self.0;
        let spawned = stored.spawned();
        let current = crew.dispatch.started(spawned.pid())?;
        let same = spawned.started().filter(|recorded| current.as_ref() == Some(*recorded));
        let ended = same.map(|_same| crew.dispatch.kill(spawned.pid())).transpose()?;
        let state = ended.map_or(RunState::Gone, Reaped::state);
        store.write(RunEnded::new(stored.identity(), state, None))?;
        Ok(Reaped { run: stored.identity(), state })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Attempts<'_, '_, S, D, C> {
    fn attempted(&self, attempt: Attempt) -> Result<Concluded, Failure> {
        let Structure { store, crew } = self.structure;
        let run = store.read(NextRun)?;
        let deadline = Deadline::after(crew.clock.now(), Structure::<S, D, C>::TIMEOUT);
        let held = crew.dispatch.dispatch(&Job::due_by(deadline), &Prompt::from(&self.job.briefing(&attempt)))?;
        let started = crew.dispatch.started(held.pid())?;
        store.write(RunSpawned::new(run, self.id, Spawned::new(held.pid(), started, deadline)))?;
        let watched = Watch::over(crew.dispatch, crew.clock).watched(held)?;
        store.write(Ending::ended(run, &watched))?;
        let retried = attempt.lapse().cloned();
        let step = self.stepped(run, attempt, watched)?;
        self.continued(retried, step)
    }

    fn continued(&self, retried: Option<Lapse>, step: Step) -> Result<Concluded, Failure> {
        match step {
            Step::Ended(ending) => Ok(Concluded { retried, ending }),
            Step::Again(again) => self.attempted(again),
        }
    }

    fn stepped(&self, run: RunId, attempt: Attempt, watched: Watched) -> Result<Step, Failure> {
        match watched {
            Watched::Overdue => Ok(Step::from(attempt.lapsed(Lapse::overdue()))),
            Watched::Finished(finished) => self.read(run, attempt.read(finished.reply()), &finished),
        }
    }

    fn read(&self, run: RunId, reading: Reading, finished: &Finished) -> Result<Step, Failure> {
        match reading {
            Reading::Parsed { directives, attempt } => Step::settled(
                attempt,
                Parsed::new(self.structure.store, directives, Provenance::Agent(run)).replied(finished.reply()),
            ),
            reading @ (Reading::Again(_) | Reading::Flailed(_)) => Ok(Step::from(reading)),
        }
    }
}

impl Reaped {
    fn state(ended: Ended) -> RunState {
        match ended {
            Ended::Killed => RunState::Killed,
            Ended::Absent => RunState::Gone,
        }
    }
}

impl Ending {
    fn ended(run: RunId, watched: &Watched) -> RunEnded {
        match watched {
            Watched::Finished(finished) => RunEnded::new(run, RunState::Replied, Some(finished.usage())),
            Watched::Overdue => RunEnded::new(run, RunState::Overdue, None),
        }
    }

    fn lines(&self, job: JobId, store: &impl Store) -> Result<Vec<Line>, Failure> {
        match self {
            Ending::Applied(application) => application.lines(store),
            Ending::Flailed(lapse) => Ok(vec![Line::Flailed(Lapsed::new(job, lapse.clone()))]),
        }
    }
}

impl Ending {
    fn flailed(&self) -> Option<&Lapse> {
        match self {
            Ending::Flailed(lapse) => Some(lapse),
            Ending::Applied(_) => None,
        }
    }
}

impl Step {
    fn settled(attempt: Attempt, applied: Result<Application, Failure>) -> Result<Step, Failure> {
        applied
            .map(|application| Step::Ended(Ending::Applied(application)))
            .or_else(|failure| failure.refusal().map(|lapse| Step::from(attempt.lapsed(lapse))))
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
        structure: &Structure<'_, S, D, C>,
        chunk: &[Unplaced],
    ) -> Result<Progress, Failure> {
        match self {
            Progress::Going(worked) => structure.job(chunk).map(|done| Progress::after(worked, done)),
            Progress::Halted(worked) => Ok(Progress::Halted(worked)),
        }
    }

    fn after(worked: Vec<Worked>, done: Worked) -> Progress {
        let ended: fn(Vec<Worked>) -> Progress =
            done.flailed.as_ref().map_or(Progress::Going, |_lapse| Progress::Halted);
        ended(worked.into_iter().chain([done]).collect())
    }

    fn worked(self) -> Vec<Worked> {
        match self {
            Progress::Going(worked) | Progress::Halted(worked) => worked,
        }
    }
}

impl Structured {
    pub(crate) fn reported(self, out: &mut impl Write) -> Result<Outcome, Failure> {
        let reaped = self.reaped.iter().map(|reaped| Line::Reaped { run: reaped.run, state: reaped.state });
        let lines: Vec<Line> =
            reaped.chain(self.worked.iter().flat_map(|worked| worked.lines.iter().cloned())).collect();
        Printed::lines(out, &lines)?;
        let jobs = Count::new(self.worked.len());
        let flailed = self
            .worked
            .into_iter()
            .find_map(|worked| worked.flailed.map(|lapse| Failure::Flailed { job: worked.job, lapse }));
        flailed.map_or(Ok(Outcome::Structured { jobs }), Err)
    }
}
