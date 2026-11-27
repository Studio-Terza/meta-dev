mod boundary;
mod hub;

use std::fmt;
use std::path::Path;

use console_ports::Heard;
use design_messages::{CheckName, FailedBefore, Failing};
use dispatch_ports::{Clock, Shell};
use file_ports::Files;
use grammar::Lines;
use invariants::Ignoring;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::boundary::Decision;
use crate::cli::failure::Failure;
pub use crate::cli::request::UnreadStep;
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Cause as Escalation, Item, Local, Target};
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
use boundary::{At, Branched, JobId, Pushed, Reminder, Report, Said};
pub use boundary::{Branch, CommitSha, Finishing};
pub use hub::Hub;
use hub::{Mergeability, Unopened, Verdict};

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
    lines: Vec<String>,
}

impl Finishing {
    pub fn heard(heard: Heard) -> Result<Finishing, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn answered<F: Files, S: Store, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let local = ground.local();
        let desk = Desk { ground, local: &local, directory: self.directory() };
        match (self.said(), self.reminder()) {
            (Said::Typed(Report::Blocked(at)), _reminder) => desk.escalated(Some(at), Escalation::Blocked),
            (Said::Typed(Report::Done(at)), _reminder) => match local.repository() {
                Some(repository) => desk.landed(at, &Hub::new(crew, self.directory(), repository)),
                None => desk.escalated(Some(at), Escalation::Unread(UnreadStep::NoRepository)),
            },
            (Said::Untyped, Reminder::First) => Decision::blocked(Untyped.to_string()),
            (Said::Untyped, Reminder::Repeated) => desk.escalated(None, Escalation::Unread(UnreadStep::Ending)),
        }
    }
}

impl<F: Files, S: Store> Desk<'_, '_, F, S> {
    fn landed<C: Clock, G: Shell>(&self, at: &At, hub: &Hub<'_, C, G>) -> Result<Uttered, Failure> {
        match at {
            At::Pull(pushed) => self.done(*pushed, &[Target::from(pushed)], hub),
            At::Branch(branched) => match hub.opened(branched, self.ground, self.local) {
                Ok(pushed) => self.done(pushed, &[Target::from(&pushed), Target::from(branched)], hub),
                Err(Unopened::Unpushed(held)) => Decision::blocked(Stale { branched, held }.to_string()),
                Err(Unopened::Unread(step)) => self.escalated(Some(at), Escalation::Unread(step)),
            },
        }
    }

    fn done<C: Clock, G: Shell>(
        &self,
        pushed: Pushed,
        passed: &[Target],
        hub: &Hub<'_, C, G>,
    ) -> Result<Uttered, Failure> {
        let at = At::Pull(pushed);
        match pushed.checked(hub, self.ground.store()) {
            Ok(Ending::Stop) => self.cleared(passed),
            Ok(Ending::Unpushed { held }) => Decision::blocked(Unpushed { pushed, held }.to_string()),
            Ok(Ending::Conflicting) => Decision::blocked(Conflicted { pushed }.to_string()),
            Ok(Ending::Failing(jobs)) => Decision::blocked(Logged::fetched(pushed, &jobs, hub).to_string()),
            Ok(Ending::Escalate(escalation)) => self.escalated(Some(&at), escalation),
            Err(step) => self.escalated(Some(&at), Escalation::Unread(step)),
        }
    }

    fn cleared(&self, passed: &[Target]) -> Result<Uttered, Failure> {
        let queued = self.local.queue().map(|file| self.ground.queue(file).map(|queue| (file, queue))).transpose()?;
        queued
            .and_then(|(file, queue)| {
                let cleared = queue.clone().cleared(passed);
                (cleared != queue).then(|| self.ground.queued(file, &cleared))
            })
            .transpose()?;
        Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::SubagentStop))))
    }

    fn escalated(&self, at: Option<&At>, escalation: Escalation) -> Result<Uttered, Failure> {
        let file = self.local.queue().ok_or(Failure::Unplaced)?;
        let item = Item::escalated(at.map(Target::from), escalation, self.directory);
        let queue = self.ground.queue(file)?.with(item);
        self.ground.queued(file, &queue)?;
        Ok(Uttered::silent(Ok(Outcome::Escalated)))
    }
}

impl Pushed {
    fn checked<C: Clock, G: Shell>(self, hub: &Hub<'_, C, G>, store: &impl Store) -> Result<Ending, UnreadStep> {
        let held = hub.head(self.pull())?;
        if held == self.head() {
            match hub.mergeable(self.pull())? {
                Mergeability::Conflicting => Ok(Ending::Conflicting),
                Mergeability::Clean | Mergeability::Unknown => Ok(self.judged(hub.settled(self.pull())?, store)),
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
    const LINES: usize = 40;

    fn fetched<C: Clock, G: Shell>(pushed: Pushed, failed: &[FailedJob], hub: &Hub<'_, C, G>) -> Logged {
        let jobs = failed.iter().map(|job| Excerpt::tailed(job.clone(), &hub.log(job.job))).collect();
        Logged { pushed, jobs }
    }
}

impl Excerpt {
    fn tailed(failed: FailedJob, log: &str) -> Excerpt {
        let whole: Vec<String> = match log.parse::<Lines>() {
            Ok(lines) => lines.into_iter().collect(),
            Err(unsplit) => match unsplit {},
        };
        let skipped = whole.len().saturating_sub(Logged::LINES);
        Excerpt { failed, lines: whole.into_iter().skip(skipped).collect() }
    }
}

impl From<&Pushed> for Target {
    fn from(pushed: &Pushed) -> Target {
        Target::Pull(pushed.pull().into_inner())
    }
}

impl From<&Branched> for Target {
    fn from(branched: &Branched) -> Target {
        Target::Branch(branched.branch().to_string())
    }
}

impl From<&At> for Target {
    fn from(at: &At) -> Target {
        match at {
            At::Pull(pushed) => Target::from(pushed),
            At::Branch(branched) => Target::from(branched),
        }
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
            write!(formatter, "\n{check} (job {job}):\n")?;
            lines.iter().try_for_each(|line| writeln!(formatter, "{line}"))
        })
    }
}
