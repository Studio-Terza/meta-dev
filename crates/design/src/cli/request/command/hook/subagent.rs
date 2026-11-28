mod boundary;
mod hub;

use std::fmt;
use std::path::Path;

use design_messages::{CheckName, CommitSha, FailedBefore, Failing};
use file_ports::Files;
use grammar::Grammar;
use grammar::lines::Lines;
use grammar::report::MissedMutant;
use invariants::Ignoring;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::{Asked, Cut, Tape};
use super::reply::{Hook, Reply};
use crate::cli::failure::Failure;
pub use crate::cli::request::UnreadStep;
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Cause as Escalation, Item, Local, Target};
use crate::{HookEvent, Outcome};
use boundary::{At, Branched, FailedJob, Pushed, Reminder, Report, Said};
pub use boundary::{Branch, Finishing};
pub use hub::Hub;
use hub::{Merge, Unopened, Verdict};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ending {
    Stop,
    Unpushed { held: CommitSha },
    MergeMain { lacking: Vec<String> },
    Failing(Vec<FailedJob>),
    Escalate(Escalation),
}

#[derive(Debug)]
struct Checked {
    ending: Ending,
    failing: Option<Failing>,
}

#[derive(Debug)]
struct Untyped;

#[derive(Debug)]
struct Unpushed {
    pushed: Pushed,
    held: CommitSha,
}

#[derive(Debug)]
struct Stalled {
    pushed: Pushed,
    lacking: Vec<String>,
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

impl Hook for Finishing {
    fn answered<F: Files + Clone, S: Store>(
        &self,
        ground: &Ground<'_, F, S>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'_>, Failure> {
        let local = ground.local();
        let directory = ground.resolved(self.directory());
        let desk = Desk { ground, local: &local, directory: &directory };
        match (self.said(), self.reminder()) {
            (Said::Typed(Report::Blocked(at)), _reminder) => desk.escalated(Some(at), Escalation::Blocked),
            (Said::Typed(Report::Done(at)), _reminder) => match local.repository() {
                Some(repository) => desk.landed(at, Hub::new(&directory, repository), tape),
                None => desk.escalated(Some(at), Escalation::Unread(UnreadStep::NoRepository)),
            },
            (Said::Untyped, Reminder::First) => Ok(Reply::Block { reason: Untyped.to_string() }),
            (Said::Untyped, Reminder::Repeated) => desk.escalated(None, Escalation::Unread(UnreadStep::Ending)),
        }
    }
}

impl<F: Files, S: Store> Desk<'_, '_, F, S> {
    fn landed(&self, at: &At, hub: Hub<'_>, tape: &Tape<'_>) -> Result<Reply<'static>, Failure> {
        match at {
            At::Pull(pushed) => self.done(*pushed, &[Target::from(pushed)], hub, tape),
            At::Branch(branched) => match Cut::settled(self.opened(branched, hub, tape))? {
                Ok(pushed) => self.done(pushed, &[Target::from(&pushed), Target::from(branched)], hub, tape),
                Err(Unopened::Unpushed(held)) => Ok(Reply::Block { reason: Stale { branched, held }.to_string() }),
                Err(Unopened::Unread(step)) => self.escalated(Some(at), Escalation::Unread(step)),
            },
        }
    }

    fn opened(&self, branched: &Branched, hub: Hub<'_>, tape: &Tape<'_>) -> Result<Pushed, Cut<Unopened>> {
        let spec = hub.cited(tape, branched)?;
        let specs = self.local.specs().ok_or(Cut::Failed(Unopened::from(UnreadStep::NoSpecs)))?;
        let title = self.ground.titled(specs, &spec).ok_or(Cut::Failed(Unopened::from(UnreadStep::Detail)))?;
        hub.opened(tape, branched, &spec, &title.to_string())
    }

    fn done(
        &self,
        pushed: Pushed,
        passed: &[Target],
        hub: Hub<'_>,
        tape: &Tape<'_>,
    ) -> Result<Reply<'static>, Failure> {
        let store = self.ground.store();
        let failed = store.read(FailedBefore::new(pushed.pull(), pushed.head())).ignoring(Ignored::Unremembered);
        let before = failed.unwrap_or_default();
        match Cut::settled(pushed.checked(tape, hub, &before))? {
            Ok(Checked { ending, failing }) => {
                let reply = self.ended(pushed, passed, hub, tape, ending)?;
                failing.map(|recorded| store.write(recorded).ignoring(Ignored::Unremembered));
                Ok(reply)
            }
            Err(step) => self.escalated(Some(&At::Pull(pushed)), Escalation::Unread(step)),
        }
    }

    fn ended(
        &self,
        pushed: Pushed,
        passed: &[Target],
        hub: Hub<'_>,
        tape: &Tape<'_>,
        ending: Ending,
    ) -> Result<Reply<'static>, Failure> {
        match ending {
            Ending::Stop => self.cleared(passed),
            Ending::Unpushed { held } => Ok(Reply::Block { reason: Unpushed { pushed, held }.to_string() }),
            Ending::MergeMain { lacking } => Ok(Reply::Block { reason: Stalled { pushed, lacking }.to_string() }),
            Ending::Failing(jobs) => {
                Ok(Reply::Block { reason: Logged::fetched(tape, pushed, &jobs, hub)?.to_string() })
            }
            Ending::Escalate(escalation) => self.escalated(Some(&At::Pull(pushed)), escalation),
        }
    }

    fn cleared(&self, passed: &[Target]) -> Result<Reply<'static>, Failure> {
        let queued = self.local.queue().map(|file| self.ground.queue(file).map(|queue| (file, queue))).transpose()?;
        queued
            .and_then(|(file, queue)| {
                let cleared = queue.clone().cleared(passed);
                (cleared != queue).then(|| self.ground.queued(file, &cleared))
            })
            .transpose()?;
        Ok(Reply::Pass(Outcome::Hooked(HookEvent::SubagentStop)))
    }

    fn escalated(&self, at: Option<&At>, escalation: Escalation) -> Result<Reply<'static>, Failure> {
        let file = self.local.queue().ok_or(Failure::Unplaced)?;
        let item = Item::escalated(at.map(Target::from), escalation, self.ground.rendered(self.directory));
        let queue = self.ground.queue(file)?.with(item);
        self.ground.queued(file, &queue)?;
        Ok(Reply::Pass(Outcome::Escalated))
    }
}

impl Pushed {
    fn checked(self, tape: &Tape<'_>, hub: Hub<'_>, before: &[CheckName]) -> Result<Checked, Cut<UnreadStep>> {
        let held = hub.head(tape, self.pull())?;
        if held == self.head() {
            match hub.mergeable(tape, self.pull())? {
                Merge::Stale => Ok(Checked::ended(Ending::MergeMain { lacking: hub.lacking(tape, self.head())? })),
                Merge::Current => {
                    hub.requested(tape, self.pull())?;
                    let judged = self.judged(hub.settled(tape, self.pull())?, before);
                    match judged.ending {
                        Ending::Stop => Ok(Checked {
                            ending: hub
                                .gated(tape, self.head())?
                                .map_or(Ending::Stop, |check| Ending::Escalate(Escalation::GatedRerun(check))),
                            failing: judged.failing,
                        }),
                        Ending::Unpushed { .. }
                        | Ending::MergeMain { .. }
                        | Ending::Failing(_)
                        | Ending::Escalate(_) => Ok(judged),
                    }
                }
            }
        } else {
            Ok(Checked::ended(Ending::Unpushed { held }))
        }
    }

    fn judged(self, verdict: Verdict, before: &[CheckName]) -> Checked {
        let jobs = verdict.failed();
        let names = jobs.iter().map(|job| job.check().clone()).collect();
        let failing = Some(Failing::new(self.pull(), self.head(), names));
        let repeated = jobs.iter().find(|job| before.contains(job.check()));
        let ending = match (repeated, jobs.first()) {
            (Some(job), _first) => Ending::Escalate(Escalation::RepeatFailure(job.check().clone())),
            (None, Some(_first)) => Ending::Failing(jobs),
            (None, None) => Ending::Stop,
        };
        Checked { ending, failing }
    }
}

impl Checked {
    fn ended(ending: Ending) -> Checked {
        Checked { ending, failing: None }
    }
}

impl Logged {
    const LINES: usize = 40;

    fn fetched(tape: &Tape<'_>, pushed: Pushed, failed: &[FailedJob], hub: Hub<'_>) -> Result<Logged, Asked> {
        let logged = failed.iter().map(|job| Ok(Excerpt::excerpted(job.clone(), &hub.log(tape, job.job())?)));
        Ok(Logged { pushed, jobs: logged.collect::<Result<_, Asked>>()? })
    }
}

impl Excerpt {
    fn excerpted(failed: FailedJob, log: &str) -> Excerpt {
        let whole: Vec<String> = Lines::from(log).into_iter().collect();
        let missed: Vec<String> = whole.iter().filter(|line| MissedMutant::read(line).is_some()).cloned().collect();
        let skipped = whole.len().saturating_sub(Logged::LINES);
        let lines = if missed.is_empty() { whole.into_iter().skip(skipped).collect() } else { missed };
        Excerpt { failed, lines }
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

impl fmt::Display for Stalled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pull = self.pushed.pull();
        write!(
            formatter,
            "Pull request #{pull} at {} is behind main or conflicts with it, so its CI cannot land it. Merge \
             origin/main, resolve any conflicts keeping both sides' behaviour, push, then end with DONE #{pull} <full \
             sha of the pushed head>.",
            self.pushed.head()
        )?;
        self.lacking.first().map_or(Ok(()), |_first| {
            write!(formatter, "\nMain has these merges the head lacks:")?;
            self.lacking.iter().try_for_each(|title| write!(formatter, "\n- {title}"))
        })
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
        self.jobs.iter().try_for_each(|Excerpt { failed, lines }| {
            write!(formatter, "\n{} (job {}):\n", failed.check(), failed.job())?;
            lines.iter().try_for_each(|line| writeln!(formatter, "{line}"))
        })
    }
}
