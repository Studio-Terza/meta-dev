use crate::Uttered;
use crate::cli::request::command::printed::Lines;
use crate::cli::request::ignored::Ignored;
use invariants::Ignoring;
use std::collections::BTreeSet;

use design_jobs::{OpenJobs, job};
use design_model::{JobId, Provenance, RunId, Segment};
use design_runs::{
    Account, Flailed, FlailedSegments, Halted, JobFlailed, JobHalted, JobOpened, JobsInterrupted, LiveRuns, NextJob,
    NextRun, Retold, RunEnded, RunSpawned, RunState, RunsOf, Spawned, StoredRun, Unplaced, Unstructured, run,
};
use design_store::{Asked, AskedEarlier, ContextNodes, Remit};
use dispatch_ports::{
    AgentRun, Clock, Deadline, Dispatch, DispatchFailure, Due, Ended, Fault, Finished, Job, Model, Prompt, Reply,
    Stall, Timeout, Watch, Watched, Watching,
};
use port_core::Held;
use store_ports::{Store, StoreFailure};
use structurer::{Attempt, Directives, Lapse, Readings, Said, SegmentRef};
use transcript_ports::{Body, SessionId};

use super::apply::{Application, Parsed};
use super::hook::answers::Reading;
use crate::Outcome;
use crate::cli::failure::{Failure, Flail, Flails};
use crate::cli::output::{Attempted, Count, Flailing, Lapsed, Line};
use crate::cli::sequel::Sequel;
use console_ports::Shown;

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
    sequel: Sequel,
    halt: Option<Halt>,
    lines: Vec<Line>,
}

#[derive(Debug)]
enum Step {
    Ended(Ending),
    Again(Attempt),
}

#[derive(Debug)]
struct Stepping<'attempts, 'structure, 'ports, S: Store, D: Dispatch, C: Clock> {
    attempts: &'attempts Attempts<'structure, 'ports, S, D, C>,
    run: RunId,
    attempt: Attempt,
}

#[derive(Debug)]
struct Applying<'attempts, 'structure, 'ports, S: Store, D: Dispatch, C: Clock> {
    attempts: &'attempts Attempts<'structure, 'ports, S, D, C>,
    run: RunId,
    finished: &'attempts Finished,
}

#[derive(Debug)]
struct Unapplied;

#[derive(Debug)]
struct RunEnding {
    run: RunId,
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

    pub fn replied(&self, prompt: &Prompt, timeout: Timeout, model: Model) -> Result<Option<Reply>, Failure> {
        let held = self.dispatch.dispatch(&Job::due_by(Deadline::after(self.clock.now(), timeout), model), prompt)?;
        Ok(Watch::over(self.dispatch, self.clock).watched(held)?.visited(Replying))
    }

    pub fn fanned(&self, reading: &Reading<'_>, timeout: Timeout) -> Result<Vec<Option<Reply>>, Failure> {
        let job = Job::due_by(Deadline::after(self.clock.now(), timeout), Model::Haiku);
        let watched = Watch::over(self.dispatch, self.clock).fanned(&job, reading.prompts())?;
        Ok(watched.iter().map(|each| each.visited(Replying)).collect())
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
            .into_iter()
            .filter(|stored| AsRef::<Deadline>::as_ref(stored.spawned()).due(now) == Due::Passed)
            .map(|stored| Reaper(self).reaped(stored))
            .collect::<Result<Vec<Reaped>, Failure>>()?;
        let interrupted = self.store.read(OpenJobs)?.iter().map(job::Open::key).copied().collect();
        self.store.write(JobsInterrupted)?;
        let unplaced = self.store.read(Unstructured)?;
        let chunks = unplaced.chunks(structurer::Job::SEGMENTS);
        let sequels = chunks.clone().skip(1).map(|_later| Sequel::WentOn).chain([Sequel::NoMoreJobs]);
        let worked = chunks.zip(sequels).try_fold(Vec::new(), |mut worked: Vec<Worked>, (chunk, sequel)| {
            let halted = worked.last().map(Worked::course).filter(|course| *course == Course::Halting);
            let done = halted.map_or_else(|| self.job(chunk, sequel, &worked).map(Some), |_| Ok(None))?;
            worked.extend(done);
            Ok::<Vec<Worked>, Failure>(worked)
        })?;
        let flailed = self.store.read(FlailedSegments)?;
        lock.end()?;
        Ok(Structured { reaped, interrupted, worked, flailed })
    }

    fn job(&self, chunk: &[Unplaced], sequel: Sequel, earlier: &[Worked]) -> Result<Worked, Failure> {
        let id = self.store.read(NextJob)?;
        let context = self.store.read(ContextNodes)?;
        let sessions = chunk.iter().map(AsRef::<SessionId>::as_ref).collect::<BTreeSet<&SessionId>>();
        let firsts = sessions
            .iter()
            .filter_map(|session| chunk.iter().find(|unplaced| AsRef::<SessionId>::as_ref(unplaced) == *session));
        let asked = firsts
            .map(|first| self.store.read(AskedEarlier::from(AsRef::<Segment>::as_ref(first))))
            .collect::<Result<Vec<Vec<Asked>>, StoreFailure>>()?
            .concat();
        let job = structurer::Job::new(Structure::<S, D, C>::said(chunk), Structure::<S, D, C>::said(&asked), context);
        let segments = chunk.iter().map(|unplaced| AsRef::<Segment>::as_ref(unplaced).clone());
        let remit = Remit::job(id, segments.clone());
        self.store.write(JobOpened::from(id).covering(segments.clone().collect()))?;
        let ended = Attempts { structure: self, id, job: &job, remit: &remit }.attempted(Attempt::First)?;
        let halt = ended.ending.halt().cloned();
        let held = halt.as_ref().and_then(Halt::flailed).map(|_lapse| JobFlailed::new(ended.run, segments.collect()));
        let halting = halt.as_ref().filter(|stopped| stopped.course() == Course::Halting);
        let retold = halting.map(|_halt| earlier.iter().filter_map(|worked| worked.retold(id)).collect());
        let told =
            halt.as_ref().map(|stopped| stopped.halted(id, sequel).holding(held).retelling(retold.unwrap_or_default()));
        told.map(|ended_job| self.store.write(ended_job)).transpose()?;
        let retried = ended.retried.iter().map(|lapse| Line::Retried(Lapsed::new(id, lapse.clone())));
        let closing = ended.ending.lines(id, self.store)?;
        let runs = self.store.read(RunsOf::from(id))?;
        let ran = runs.iter().map(|ran| Line::Ran(Attempted::new(ran.identity(), id, ran.state(), ran.usage())));
        let lines = retried.chain(closing).chain(ran).collect();
        Ok(Worked { job: id, sequel, halt, lines })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Structure<'_, S, D, C> {
    fn said<Spoken: AsRef<Segment> + AsRef<Body>>(spoken: &[Spoken]) -> Vec<Said> {
        spoken
            .iter()
            .map(|heard| {
                Said::new(SegmentRef::from(AsRef::<Segment>::as_ref(heard)), AsRef::<Body>::as_ref(heard).clone())
            })
            .collect()
    }
}

impl<S: Store, D: Dispatch, C: Clock> Reaper<'_, '_, S, D, C> {
    fn reaped(&self, stored: StoredRun) -> Result<Reaped, Failure> {
        let Structure { store, crew } = self.0;
        let spawned = stored.spawned();
        let current = crew.dispatch.started(spawned.pid())?;
        let same = spawned.started().filter(|recorded| current.as_ref() == Some(*recorded));
        let ended = same.map(|_same| crew.dispatch.kill(spawned.pid())).transpose()?;
        let ending = ended.map_or(run::Ended::Gone, Reaped::ended);
        let run = stored.identity();
        store.write(stored.reaped(ending))?;
        let state = RunState::from(ending);
        Ok(Reaped { run, state })
    }
}

impl<S: Store, D: Dispatch, C: Clock> Attempts<'_, '_, S, D, C> {
    fn attempted(&self, attempt: Attempt) -> Result<Concluded, Failure> {
        let Structure { store, crew } = self.structure;
        let run = store.read(NextRun)?;
        let deadline = Deadline::after(crew.clock.now(), Structure::<S, D, C>::TIMEOUT);
        let record = RunRecord { structure: self.structure, job: self.id, run };
        let dispatched =
            crew.dispatch.dispatch(&Job::due_by(deadline, Model::Haiku), &Prompt::from(&self.job.briefing(&attempt)));
        let watched = record.spawned(dispatched, deadline)?;
        let retried = attempt.lapse().cloned();
        let step = watched.visited(Stepping { attempts: self, run, attempt })?;
        self.continued(run, retried, step)
    }

    fn continued(&self, run: RunId, retried: Option<Lapse>, step: Step) -> Result<Concluded, Failure> {
        match step {
            Step::Ended(ending) => Ok(Concluded { run, retried, ending }),
            Step::Again(again) => self.attempted(again),
        }
    }
}

impl<S: Store, D: Dispatch, C: Clock> Watching for Stepping<'_, '_, '_, S, D, C> {
    type Output = Result<Step, Failure>;

    fn finished(self, finished: &Finished) -> Result<Step, Failure> {
        self.attempt.read(finished.reply()).visited(Applying { attempts: self.attempts, run: self.run, finished })
    }

    fn stalled(self, stall: &Stall) -> Result<Step, Failure> {
        Ok(Step::Ended(Ending::Halted(Halt::Stalled(stall.clone()))))
    }
}

impl<S: Store, D: Dispatch, C: Clock> Readings for Applying<'_, '_, '_, S, D, C> {
    type Output = Result<Step, Failure>;

    fn parsed(self, directives: Directives, attempt: Attempt) -> Result<Step, Failure> {
        let Attempts { structure, remit, .. } = self.attempts;
        Step::settled(
            attempt,
            Parsed::new(structure.store, directives, Provenance::Agent(self.run), (*remit).clone())
                .replied(self.finished.reply()),
        )
    }

    fn again(self, attempt: Attempt) -> Result<Step, Failure> {
        Ok(Unapplied.again(attempt))
    }

    fn flailed(self, lapse: Lapse) -> Result<Step, Failure> {
        Ok(Unapplied.flailed(lapse))
    }
}

impl Readings for Unapplied {
    type Output = Step;

    fn parsed(self, _directives: Directives, attempt: Attempt) -> Step {
        Step::Again(attempt)
    }

    fn again(self, attempt: Attempt) -> Step {
        Step::Again(attempt)
    }

    fn flailed(self, lapse: Lapse) -> Step {
        Step::Ended(Ending::Halted(Halt::Flailed(lapse)))
    }
}

#[derive(Clone, Copy, Debug)]
struct Replying;

impl Watching for Replying {
    type Output = Option<Reply>;

    fn finished(self, finished: &Finished) -> Option<Reply> {
        Some(finished.reply().clone())
    }

    fn stalled(self, _stall: &Stall) -> Option<Reply> {
        None
    }
}

impl Watching for RunEnding {
    type Output = RunEnded;

    fn finished(self, finished: &Finished) -> RunEnded {
        RunEnded::finished(self.run, finished)
    }

    fn stalled(self, stall: &Stall) -> RunEnded {
        RunEnded::new(self.run, RunState::from(stall), stall.usage())
    }
}

impl<S: Store, D: Dispatch, C: Clock> RunRecord<'_, '_, S, D, C> {
    fn ended(&self, held: Held<'_, AgentRun>, deadline: Deadline) -> Result<Watched, Failure> {
        let Structure { store, crew } = self.structure;
        let looked = crew.dispatch.started(held.pid());
        let started = looked.as_ref().ignoring(Ignored::Unlooked).cloned().flatten();
        store.write(RunSpawned::new(self.run, self.job, Spawned::new(held.pid(), started, deadline)))?;
        let watched = Self::watched(looked.and_then(|_started| Watch::over(crew.dispatch, crew.clock).watched(held)));
        store.write(watched.visited(RunEnding { run: self.run }))?;
        Ok(watched)
    }

    fn spawned(
        &self,
        dispatched: Result<Held<'_, AgentRun>, DispatchFailure>,
        deadline: Deadline,
    ) -> Result<Watched, Failure> {
        match dispatched {
            Ok(held) => self.ended(held, deadline),
            Err(failure) => Ok(Watched::Stalled(Stall::Unspawned(Fault::from(failure)))),
        }
    }

    fn watched(watching: Result<Watched, DispatchFailure>) -> Watched {
        match watching {
            Ok(watched) => watched,
            Err(failure) => Watched::Stalled(Stall::Failed(Fault::from(failure))),
        }
    }
}

impl Reaped {
    fn ended(ended: Ended) -> run::Ended {
        match ended {
            Ended::Killed => run::Ended::Killed,
            Ended::Absent => run::Ended::Gone,
        }
    }
}

impl Ending {
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
    fn halted(&self, job: JobId, sequel: Sequel) -> JobHalted {
        JobHalted::new(job, Halted::from(self), Account::told(&self.clone().failure(job, sequel)))
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

    fn failure(self, job: JobId, sequel: Sequel) -> Failure {
        match self {
            Halt::Flailed(lapse) => {
                Failure::Flailed { flails: Flails::new(Flail::new(job, lapse), Vec::new()), sequel }
            }
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
            .or_else(|failure| failure.refusal().map(|lapse| attempt.lapsed(lapse).visited(Unapplied)))
    }
}

impl Worked {
    fn course(&self) -> Course {
        self.halt.as_ref().map_or(Course::Onward, Halt::course)
    }

    fn retold(&self, stalled: JobId) -> Option<Retold> {
        let flailed = self.halt.as_ref().and_then(Halt::flailed).cloned().map(Halt::Flailed);
        flailed
            .map(|halt| Retold::new(self.job, Account::told(&halt.failure(self.job, Sequel::Halted { job: stalled }))))
    }
}

impl Structured {
    pub(crate) fn reported(self) -> Uttered {
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
        let shown = Shown::said(Lines::from(&lines));
        Uttered::new(shown, self.concluded().map(|jobs| Outcome::Structured { jobs }))
    }

    pub(crate) fn concluded(self) -> Result<Count, Failure> {
        let jobs = Count::new(self.worked.len());
        let stalled = self.worked.iter().find_map(|worked| {
            let halting = worked.halt.clone().filter(|halt| halt.course() == Course::Halting);
            halting.map(|halt| halt.failure(worked.job, worked.sequel))
        });
        let flails: Vec<Flail> = self
            .worked
            .iter()
            .filter_map(|worked| {
                worked.halt.as_ref().and_then(Halt::flailed).map(|lapse| Flail::new(worked.job, lapse.clone()))
            })
            .collect();
        let sequel = self
            .worked
            .iter()
            .rev()
            .find_map(|worked| worked.halt.as_ref().and_then(Halt::flailed).map(|_lapse| worked.sequel));
        let flailed = flails.split_first().zip(sequel).map(|((first, rest), followed)| Failure::Flailed {
            flails: Flails::new(first.clone(), rest.to_vec()),
            sequel: followed,
        });
        stalled.or(flailed).map_or(Ok(jobs), Err)
    }
}
