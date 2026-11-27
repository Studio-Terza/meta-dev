mod boundary;

use std::fmt;
use std::path::Path;

use console_ports::Heard;
use design_messages::{CheckName, FailedBefore, Failing};
use dispatch_ports::{Clock, Dispatch, Shell};
use file_ports::Files;
use invariants::Ignoring;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::boundary::Decision;
use crate::cli::failure::Failure;
use crate::cli::request::Local;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
use boundary::{At, Branched, JobId, Mergeability, Pushed, Reminder, Report, Said, Tail, Unopened, Verdict};
pub use boundary::{Branch, CommitSha, Finishing, Hub};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ending {
    Stop,
    Unpushed { held: CommitSha },
    Conflicting,
    Failing(Vec<FailedJob>),
    Escalate(Escalation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedJob {
    check: CheckName,
    job: JobId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Escalation {
    Blocked,
    RepeatFailure(CheckName),
    Unread(UnreadStep),
}

#[derive(strum::Display, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum UnreadStep {
    Ending,
    Head,
    Mergeable,
    Watch,
    Overdue,
    Checks,
    NoChecks,
    NoRepository,
    NoSpecs,
    Remote,
    Commits,
    Spec,
    Detail,
    Open,
}

#[derive(Debug)]
struct Untyped;

#[derive(Debug)]
struct Unpushed {
    pushed: Pushed,
    held: CommitSha,
}

#[derive(Debug)]
struct Conflicted {
    pushed: Pushed,
}

#[derive(Debug)]
struct Desk<'ground, 'run, F: Files, S: Store> {
    ground: &'ground Ground<'run, F, S>,
    local: &'ground Local,
    directory: &'ground Path,
}

#[derive(Debug)]
struct Stale<'report> {
    branched: &'report Branched,
    held: Option<CommitSha>,
}

#[derive(Debug)]
struct Logged {
    pushed: Pushed,
    jobs: Vec<Excerpt>,
}

#[derive(Debug)]
struct Excerpt {
    failed: FailedJob,
    lines: Tail,
}

#[derive(Debug)]
struct Escalated<'finishing> {
    escalation: Escalation,
    at: Option<At>,
    directory: &'finishing Path,
}

impl Finishing {
    pub fn heard(heard: Heard) -> Result<Finishing, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub(crate) fn answered<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let local = ground.local();
        let desk = Desk { ground, local: &local, directory: self.directory() };
        match (self.said(), self.reminder()) {
            (Said::Typed(Report::Blocked(at)), _reminder) => desk.escalated(Some(at.clone()), Escalation::Blocked),
            (Said::Typed(Report::Done(at)), _reminder) => match local.repository() {
                Some(repository) => desk.landed(at, &Hub::new(crew, self.directory(), repository)),
                None => desk.escalated(Some(at.clone()), Escalation::Unread(UnreadStep::NoRepository)),
            },
            (Said::Untyped, Reminder::First) => Decision::blocked(Untyped.to_string()),
            (Said::Untyped, Reminder::Repeated) => desk.escalated(None, Escalation::Unread(UnreadStep::Ending)),
        }
    }
}

impl<F: Files, S: Store> Desk<'_, '_, F, S> {
    fn landed<D: Dispatch, C: Clock, G: Shell>(&self, at: &At, hub: &Hub<'_, D, C, G>) -> Result<Uttered, Failure> {
        match at {
            At::Pull(pushed) => self.done(*pushed, hub),
            At::Branch(branched) => match hub.opened(branched, self.ground, self.local) {
                Ok(pushed) => self.done(pushed, hub),
                Err(Unopened::Unpushed(held)) => Decision::blocked(Stale { branched, held }.to_string()),
                Err(Unopened::Unread(step)) => {
                    self.escalated(Some(At::Branch(branched.clone())), Escalation::Unread(step))
                }
            },
        }
    }

    fn done<D: Dispatch, C: Clock, G: Shell>(
        &self,
        pushed: Pushed,
        hub: &Hub<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        match pushed.checked(hub, self.ground.store()) {
            Ok(Ending::Stop) => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::SubagentStop)))),
            Ok(Ending::Unpushed { held }) => Decision::blocked(Unpushed { pushed, held }.to_string()),
            Ok(Ending::Conflicting) => Decision::blocked(Conflicted { pushed }.to_string()),
            Ok(Ending::Failing(jobs)) => Decision::blocked(Logged::fetched(pushed, &jobs, hub).to_string()),
            Ok(Ending::Escalate(escalation)) => self.escalated(Some(At::Pull(pushed)), escalation),
            Err(step) => self.escalated(Some(At::Pull(pushed)), Escalation::Unread(step)),
        }
    }

    fn escalated(&self, at: Option<At>, escalation: Escalation) -> Result<Uttered, Failure> {
        let queue = self.local.queue().ok_or(Failure::Unplaced)?;
        let line = Escalated { escalation, at, directory: self.directory };
        self.ground.queued(queue, line).map_err(Failure::Unqueued)?;
        Ok(Uttered::silent(Ok(Outcome::Escalated)))
    }
}

impl Pushed {
    fn checked<D: Dispatch, C: Clock, G: Shell>(
        self,
        hub: &Hub<'_, D, C, G>,
        store: &impl Store,
    ) -> Result<Ending, UnreadStep> {
        let held = hub.head(self.pull())?;
        if held == self.head() {
            match hub.mergeable(self.pull())? {
                Mergeability::Conflicting => Ok(Ending::Conflicting),
                Mergeability::Clean | Mergeability::Unknown => {
                    hub.registered(self.pull())?;
                    hub.watched(self.pull())?;
                    Ok(self.judged(hub.checks(self.pull())?, store))
                }
            }
        } else {
            Ok(Ending::Unpushed { held })
        }
    }

    fn judged(self, verdict: Verdict, store: &impl Store) -> Ending {
        let before: Vec<CheckName> =
            store.read(FailedBefore::from(self.pull())).ignoring(Ignored::Unremembered).unwrap_or_default();
        let jobs = verdict.failed();
        let names = jobs.iter().map(|job| job.check.clone()).collect();
        store.write(Failing::new(self.pull(), names)).ignoring(Ignored::Unremembered);
        let repeated = jobs.iter().find(|job| before.contains(&job.check));
        match (repeated, jobs.first()) {
            (Some(job), _first) => Ending::Escalate(Escalation::RepeatFailure(job.check.clone())),
            (None, Some(_first)) => Ending::Failing(jobs),
            (None, None) => Ending::Stop,
        }
    }
}

impl FailedJob {
    pub fn new(check: CheckName, job: JobId) -> FailedJob {
        FailedJob { check, job }
    }
}

impl Logged {
    fn fetched<D: Dispatch, C: Clock, G: Shell>(
        pushed: Pushed,
        failed: &[FailedJob],
        hub: &Hub<'_, D, C, G>,
    ) -> Logged {
        let jobs = failed.iter().map(|job| Excerpt { failed: job.clone(), lines: hub.log(job.job) }).collect();
        Logged { pushed, jobs }
    }
}

impl fmt::Display for Untyped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "End your turn with exactly one line and nothing else: DONE <branch> <full sha> once that commit is the \
             pushed head of a branch with no pull request yet, DONE #<pr> <full sha> once the pull request's head is \
             that commit, or BLOCKED #<pr> <full sha> when you cannot go on.",
        )
    }
}

impl fmt::Display for Conflicted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pull = self.pushed.pull();
        write!(
            formatter,
            "Pull request #{pull} at {} conflicts with main. Merge origin/main, resolve the conflicts, push, then end \
             with DONE #{pull} <full sha of the pushed head>.",
            self.pushed.head()
        )
    }
}

impl fmt::Display for Stale<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Stale { branched, held } = self;
        let branch = branched.branch();
        match held {
            Some(remote) => write!(formatter, "Branch {branch} has remote head {remote}, not {}.", branched.head()),
            None => write!(formatter, "Branch {branch} is not on the remote."),
        }?;
        write!(formatter, " Push the commit, then end with DONE {branch} <full sha of the pushed head>.")
    }
}

impl fmt::Display for At {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            At::Pull(pushed) => write!(formatter, "#{} {}", pushed.pull(), pushed.head()),
            At::Branch(branched) => write!(formatter, "{} {}", branched.branch(), branched.head()),
        }
    }
}

impl fmt::Display for Unpushed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Unpushed { pushed, held } = self;
        let pull = pushed.pull();
        write!(
            formatter,
            "Pull request #{pull} has head {held}, not {}. Push the commit, then end with DONE #{pull} <full sha of \
             the pushed head>.",
            pushed.head()
        )
    }
}

impl fmt::Display for Logged {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pull = self.pushed.pull();
        writeln!(
            formatter,
            "CI failed on #{pull} at {}. Fix these jobs, push, then end with DONE #{pull} <full sha>.",
            self.pushed.head()
        )?;
        self.jobs.iter().try_for_each(|Excerpt { failed: FailedJob { check, job }, lines }| {
            write!(formatter, "\n{check} (job {job}):\n{lines}")
        })
    }
}

impl fmt::Display for Escalation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Escalation::Blocked => formatter.write_str("blocked"),
            Escalation::RepeatFailure(check) => write!(formatter, "repeat failure of {check}"),
            Escalation::Unread(step) => write!(formatter, "unread {step}"),
        }
    }
}

impl fmt::Display for Escalated<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "- [agent outcome] {}", self.escalation)?;
        self.at.iter().try_for_each(|at| write!(formatter, " on {at}"))?;
        write!(formatter, ", from {}", self.directory.display())
    }
}
