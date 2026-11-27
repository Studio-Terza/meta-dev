mod boundary;
mod hub;

use std::fmt;
use std::path::Path;

use console_ports::Heard;
use design_messages::{CheckName, FailedBefore, Failing};
use dispatch_ports::{Clock, Dispatch, Shell};
use file_ports::Files;
use grammar_lines::Lines;
use invariants::Ignoring;
use newtype::Wrapping;
use store_ports::Store;

use super::super::Ground;
use super::super::crew::Crew;
use super::boundary::Decision;
use crate::cli::failure::Failure;
use crate::cli::request::ignored::Ignored;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub use boundary::Finishing;
use boundary::{CommitSha, JobId, Pushed, Reminder, Report, Said};
use hub::{Hub, Verdict};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ending {
    Stop,
    Unpushed { held: CommitSha },
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
    Watch,
    Overdue,
    Checks,
}

#[derive(Debug)]
struct Untyped;

#[derive(Debug)]
struct Unpushed {
    pushed: Pushed,
    held: CommitSha,
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

#[derive(Debug)]
struct Escalated<'finishing> {
    escalation: Escalation,
    pushed: Option<Pushed>,
    directory: &'finishing Path,
}

impl Finishing {
    pub fn heard(heard: Heard) -> Result<Finishing, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let hub = Hub::new(crew, self.directory());
        match (self.said(), self.reminder()) {
            (Said::Typed(report), _reminder) => self.ended(*report, &report.ending(&hub, ground.store()), ground, &hub),
            (Said::Untyped, Reminder::First) => Decision::blocked(Untyped.to_string()),
            (Said::Untyped, Reminder::Repeated) => self.escalated(ground, None, Escalation::Unread(UnreadStep::Ending)),
        }
    }

    fn ended<F: Files, S: Store, D: Dispatch, C: Clock, G: Shell>(
        &self,
        report: Report,
        ending: &Ending,
        ground: &Ground<'_, F, S>,
        hub: &Hub<'_, D, C, G>,
    ) -> Result<Uttered, Failure> {
        let pushed = report.pushed();
        match ending {
            Ending::Stop => Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::SubagentStop)))),
            Ending::Unpushed { held } => Decision::blocked(Unpushed { pushed, held: *held }.to_string()),
            Ending::Failing(jobs) => Decision::blocked(Logged::fetched(pushed, jobs, hub).to_string()),
            Ending::Escalate(escalation) => self.escalated(ground, Some(pushed), escalation.clone()),
        }
    }

    fn escalated<F: Files, S: Store>(
        &self,
        ground: &Ground<'_, F, S>,
        pushed: Option<Pushed>,
        escalation: Escalation,
    ) -> Result<Uttered, Failure> {
        ground.queued(Escalated { escalation, pushed, directory: self.directory() }).map_err(Failure::Unqueued)?;
        Ok(Uttered::silent(Ok(Outcome::Escalated)))
    }
}

impl Report {
    fn ending<D: Dispatch, C: Clock, G: Shell>(self, hub: &Hub<'_, D, C, G>, store: &impl Store) -> Ending {
        match self {
            Report::Blocked(_pushed) => Ending::Escalate(Escalation::Blocked),
            Report::Done(pushed) => match pushed.checked(hub, store) {
                Ok(ending) => ending,
                Err(step) => Ending::Escalate(Escalation::Unread(step)),
            },
        }
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
            hub.watched(self.pull())?;
            Ok(self.judged(hub.checks(self.pull())?, store))
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

    fn fetched<D: Dispatch, C: Clock, G: Shell>(
        pushed: Pushed,
        failed: &[FailedJob],
        hub: &Hub<'_, D, C, G>,
    ) -> Logged {
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

impl fmt::Display for Untyped {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "End your turn with exactly one line and nothing else: DONE #<pr> <full sha> once the pull request's head \
             is that commit, or BLOCKED #<pr> <full sha> when you cannot go on.",
        )
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
        self.pushed.iter().try_for_each(|pushed| write!(formatter, " on #{} {}", pushed.pull(), pushed.head()))?;
        write!(formatter, ", from {}", self.directory.display())
    }
}
