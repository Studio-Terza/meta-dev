use std::io::Write;

use design_model::{JobId, Provenance, RunId, Segment};
use design_store::{
    Account, ContextNodes, Flailed, FlailedSegments, Halted, JobFlailed, JobHalted, JobOpened, JobsInterrupted,
    LiveRuns, NextJob, NextRun, OpenJobs, Remit, RunEnded, RunSpawned, RunState, RunsOf, Spawned, StoredRun, Unplaced,
    Unstructured,
};
use dispatch_ports::{
    AgentRun, Clock, Deadline, Dispatch, Due, Ended, Fault, Finished, Job, Prompt, Stall, Timeout, Watch, Watched,
};
use port_core::Held;
use store_ports::Store;
use structurer::{Attempt, Lapse, Reading, Said, SegmentRef};
use transcript_ports::Body;

use super::Printed;
use super::apply::{Application, Parsed};
use crate::Outcome;
use crate::cli::failure::{Failure, Flail, Flails};
use crate::cli::output::{Attempted, Count, Flailing, Lapsed, Line};

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
    remit: &'structure Remit,
}

#[derive(Debug)]
struct RunRecord<'structure, 'ports, S: Store, D: Dispatch, C: Clock> {
    structure: &'structure Structure<'ports, S, D, C>,
    job: JobId,
    run: RunId,
}

#[derive(Debug)]
enum Ending {
    Applied(Application),
    Halted(Halt),
}

#[derive(Clone, Debug)]
enum Halt {
    Flailed(Lapse),
    Stalled(Stall),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Course {
    Onward,
    Halting,
}

#[derive(Debug)]
struct Concluded {
    run: RunId,
    retried: Option<Lapse>,
    ending: Ending,
}

#[derive(Debug)]
struct Worked {
    job: JobId,
    halt: Option<Halt>,
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
    interrupted: Vec<JobId>,
    worked: Vec<Worked>,
    flailed: Vec<Flailed>,
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
        let interrupted = self.store.read(OpenJobs)?;
        self.store.write(JobsInterrupted)?;
        let unplaced = self.store.read(Unstructured)?;
        let progress = unplaced
            .chunks(structurer::Job::SEGMENTS)
            .try_fold(Progress::Going(Vec::new()), |progress, chunk| progress.next(self, chunk))?;
        let flailed = self.store.read(FlailedSegments)?;
        lock.end()?;
        Ok(Structured { reaped, interrupted, worked: progress.worked(), flailed })
    }

    fn job(&self, chunk: &[Unplaced]) -> Result<Worked, Failure> {
        let id = self.store.read(NextJob)?;
        let context = self.store.read(ContextNodes)?;
        let said = chunk.iter().map(|unplaced| {
            Said::new(SegmentRef::from(AsRef::<Segment>::as_ref(unplaced)), AsRef::<Body>::as_ref(unplaced).clone())
        });
        let job = structurer::Job::new(said.collect(), context);
        let segments = chunk.iter().map(|unplaced| AsRef::<Segment>::as_ref(unplaced).clone());
        let remit = Remit::job(id, segments.clone());
        self.store.write(JobOpened::from(id))?;
        let ended = Attempts { structure: self, id, job: &job, remit: &remit }.attempted(Attempt::First)?;
        let halted = ended.ending.halt().map(|halt| halt.halted(id));
        halted.map(|halted| self.store.write(halted)).transpose()?;
        let flailed = ended.ending.halt().and_then(Halt::flailed);
        let held = flailed.map(|_lapse| JobFlailed::new(ended.run, segments.collect()));
        held.map(|flailed| self.store.write(flailed)).transpose()?;
        let retried = ended.retried.iter().map(|lapse| Line::Retried(Lapsed::new(id, lapse.clone())));
        let closing = ended.ending.lines(id, self.store)?;
        let runs = self.store.read(RunsOf::from(id))?;
        let ran = runs.iter().map(|ran| Line::Ran(Attempted::new(ran.identity(), id, ran.state(), ran.usage())));
        let lines = retried.chain(closing).chain(ran).collect();
        Ok(Worked { job: id, halt: ended.ending.halt().cloned(), lines })
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
        let record = RunRecord { structure: self.structure, job: self.id, run };
        let watched =
            crew.dispatch.dispatch(&Job::due_by(deadline), &Prompt::from(&self.job.briefing(&attempt))).map_or_else(
                |failure| Ok(Watched::Stalled(Stall::Unspawned(Fault::from(failure)))),
                |held| record.ended(held, deadline),
            )?;
        let retried = attempt.lapse().cloned();
        let step = self.stepped(run, attempt, watched)?;
        self.continued(run, retried, step)
    }

    fn continued(&self, run: RunId, retried: Option<Lapse>, step: Step) -> Result<Concluded, Failure> {
        match step {
            Step::Ended(ending) => Ok(Concluded { run, retried, ending }),
            Step::Again(again) => self.attempted(again),
        }
    }

    fn stepped(&self, run: RunId, attempt: Attempt, watched: Watched) -> Result<Step, Failure> {
        match watched {
            Watched::Stalled(stall) => Ok(Step::Ended(Ending::Halted(Halt::Stalled(stall)))),
            Watched::Finished(finished) => self.read(run, attempt.read(finished.reply()), &finished),
        }
    }

    fn read(&self, run: RunId, reading: Reading, finished: &Finished) -> Result<Step, Failure> {
        match reading {
            Reading::Parsed { directives, attempt } => Step::settled(
                attempt,
                Parsed::new(self.structure.store, directives, Provenance::Agent(run), self.remit.clone())
                    .replied(finished.reply()),
            ),
            reading @ (Reading::Again(_) | Reading::Flailed(_)) => Ok(Step::from(reading)),
        }
    }
}

impl<S: Store, D: Dispatch, C: Clock> RunRecord<'_, '_, S, D, C> {
    fn ended(&self, held: Held<'_, AgentRun>, deadline: Deadline) -> Result<Watched, Failure> {
        let Structure { store, crew } = self.structure;
        let looked = crew.dispatch.started(held.pid());
        let started = looked.as_ref().ok().cloned().flatten();
        store.write(RunSpawned::new(self.run, self.job, Spawned::new(held.pid(), started, deadline)))?;
        let watched = looked
            .and_then(|_started| Watch::over(crew.dispatch, crew.clock).watched(held))
            .unwrap_or_else(|failure| Watched::Stalled(Stall::Failed(Fault::from(failure))));
        store.write(Ending::ended(self.run, &watched))?;
        Ok(watched)
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
            Watched::Finished(finished) => RunEnded::finished(run, finished),
            Watched::Stalled(stall) => RunEnded::new(run, RunState::from(stall), stall.usage()),
        }
    }

    fn lines(&self, job: JobId, store: &impl Store) -> Result<Vec<Line>, Failure> {
        match self {
            Ending::Applied(application) => application.lines(store),
            Ending::Halted(Halt::Flailed(lapse)) => Ok(vec![Line::Flailed(Lapsed::new(job, lapse.clone()))]),
            Ending::Halted(Halt::Stalled(_stall)) => Ok(Vec::new()),
        }
    }
}

impl Ending {
    fn halt(&self) -> Option<&Halt> {
        match self {
            Ending::Halted(halt) => Some(halt),
            Ending::Applied(_) => None,
        }
    }
}

impl Halt {
    fn halted(&self, job: JobId) -> JobHalted {
        JobHalted::new(job, Halted::from(self), Account::told(&self.clone().failure(job)))
    }

    fn flailed(&self) -> Option<&Lapse> {
        match self {
            Halt::Flailed(lapse) => Some(lapse),
            Halt::Stalled(_) => None,
        }
    }

    fn course(&self) -> Course {
        match self {
            Halt::Flailed(_) => Course::Onward,
            Halt::Stalled(_) => Course::Halting,
        }
    }

    fn failure(self, job: JobId) -> Failure {
        match self {
            Halt::Flailed(lapse) => Failure::Flailed(Flails::new(Flail::new(job, lapse), Vec::new())),
            Halt::Stalled(stall) => Failure::Stalled { job, stall },
        }
    }
}

impl From<&Halt> for Halted {
    fn from(halt: &Halt) -> Halted {
        match halt {
            Halt::Flailed(_) => Halted::Flailed,
            Halt::Stalled(_) => Halted::Stalled,
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
            Reading::Flailed(lapse) => Step::Ended(Ending::Halted(Halt::Flailed(lapse))),
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
            Progress::Going(worked) => Ok(Progress::after(worked, structure.job(chunk)?)),
            Progress::Halted(worked) => Ok(Progress::Halted(worked)),
        }
    }

    fn after(worked: Vec<Worked>, done: Worked) -> Progress {
        let course = done.halt.as_ref().map_or(Course::Onward, Halt::course);
        course.progress(worked.into_iter().chain([done]).collect())
    }

    fn worked(self) -> Vec<Worked> {
        match self {
            Progress::Going(worked) | Progress::Halted(worked) => worked,
        }
    }
}

impl Course {
    fn progress(self, worked: Vec<Worked>) -> Progress {
        match self {
            Course::Onward => Progress::Going(worked),
            Course::Halting => Progress::Halted(worked),
        }
    }
}

impl Structured {
    pub(crate) fn reported(self, out: &mut impl Write) -> Result<Outcome, Failure> {
        let reaped = self
            .reaped
            .iter()
            .map(|reaped| Line::Reaped { run: reaped.run, state: reaped.state })
            .chain(self.interrupted.iter().copied().map(Line::Interrupted));
        let held = self.flailed.iter().map(|flailed| {
            Line::FlailedSegment(Flailing::new(
                SegmentRef::from(AsRef::<Segment>::as_ref(flailed)),
                flailed.job(),
                flailed.run(),
            ))
        });
        let lines: Vec<Line> =
            reaped.chain(self.worked.iter().flat_map(|worked| worked.lines.iter().cloned())).chain(held).collect();
        Printed::lines(out, &lines)?;
        self.concluded().map(|jobs| Outcome::Structured { jobs })
    }

    pub(crate) fn concluded(self) -> Result<Count, Failure> {
        let jobs = Count::new(self.worked.len());
        let stalled = self.worked.iter().find_map(|worked| {
            let halting = worked.halt.clone().filter(|halt| halt.course() == Course::Halting);
            halting.map(|halt| halt.failure(worked.job))
        });
        let flails: Vec<Flail> = self
            .worked
            .iter()
            .filter_map(|worked| {
                worked.halt.as_ref().and_then(Halt::flailed).map(|lapse| Flail::new(worked.job, lapse.clone()))
            })
            .collect();
        let flailed =
            flails.split_first().map(|(first, rest)| Failure::Flailed(Flails::new(first.clone(), rest.to_vec())));
        stalled.or(flailed).map_or(Ok(jobs), Err)
    }
}
