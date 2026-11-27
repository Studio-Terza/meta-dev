mod boundary;

use std::ops::Not;
use std::path::Path;

use design_messages::{CheckName, PullNumber};
use dispatch_ports::{Clock, Dispatch, Ran, Shell, Timeout};
use serde::Deserialize;

use super::super::super::crew::Crew;
use super::boundary::{CommitSha, JobId};
use super::{FailedJob, UnreadStep};

#[derive(Debug)]
pub struct Hub<'crew, D: Dispatch, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, D, C, G>,
    directory: &'crew Path,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Green,
    Red(Vec<FailedJob>),
}

#[derive(Debug, Deserialize)]
struct Head {
    #[serde(rename = "headRefOid")]
    head: CommitSha,
}

#[derive(Debug, Deserialize)]
struct Check {
    name: CheckName,
    bucket: Bucket,
    link: Link,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Bucket {
    Pass,
    Skipping,
    Fail,
    Cancel,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Passed,
    Failed,
    Waiting,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "String")]
struct Link(Option<JobId>);

impl<'crew, D: Dispatch, C: Clock, G: Shell> Hub<'crew, D, C, G> {
    const PROGRAM: &'static str = "gh";
    const QUICK: Timeout = Timeout::new(60);
    const WATCH: Timeout = Timeout::new(20 * 60);
    const UNLOGGED: &'static str = "(the log could not be read)";

    pub fn new(crew: &'crew Crew<'crew, D, C, G>, directory: &'crew Path) -> Hub<'crew, D, C, G> {
        Hub { crew, directory }
    }

    pub(crate) fn head(&self, pull: PullNumber) -> Result<CommitSha, UnreadStep> {
        let ran = self.ran(&["pr", "view", &pull.to_string(), "--json", "headRefOid"], Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(UnreadStep::Head)?;
        serde_json::from_str::<Head>(printed.as_ref()).map(|read| read.head).map_err(|_unread| UnreadStep::Head)
    }

    pub(crate) fn watched(&self, pull: PullNumber) -> Result<(), UnreadStep> {
        match self.ran(&["pr", "checks", &pull.to_string(), "--watch", "--fail-fast"], Self::WATCH) {
            Some(Ran::Succeeded(_) | Ran::Failed(_)) => Ok(()),
            Some(Ran::Overdue) => Err(UnreadStep::Overdue),
            None => Err(UnreadStep::Watch),
        }
    }

    pub(crate) fn checks(&self, pull: PullNumber) -> Result<Verdict, UnreadStep> {
        let ran = self.ran(&["pr", "checks", &pull.to_string(), "--json", "name,bucket,link"], Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::printed).ok_or(UnreadStep::Checks)?;
        let checks: Vec<Check> = serde_json::from_str(printed.as_ref()).map_err(|_unread| UnreadStep::Checks)?;
        Verdict::read(&checks).ok_or(UnreadStep::Checks)
    }
}

impl Verdict {
    fn read(checks: &[Check]) -> Option<Verdict> {
        let failed = checks
            .iter()
            .filter(|check| check.bucket.state() == State::Failed)
            .map(Check::failed)
            .collect::<Option<Vec<FailedJob>>>()?;
        let settled = checks.is_empty().not() && checks.iter().all(|check| check.bucket.state() == State::Passed);
        match failed.first() {
            Some(_first) => Some(Verdict::Red(failed)),
            None => settled.then_some(Verdict::Green),
        }
    }

    pub(crate) fn failed(self) -> Vec<FailedJob> {
        match self {
            Verdict::Green => Vec::new(),
            Verdict::Red(jobs) => jobs,
        }
    }
}

impl Check {
    fn failed(&self) -> Option<FailedJob> {
        self.link.0.map(|job| FailedJob::new(self.name.clone(), job))
    }
}

impl Bucket {
    fn state(self) -> State {
        match self {
            Bucket::Pass | Bucket::Skipping => State::Passed,
            Bucket::Fail | Bucket::Cancel => State::Failed,
            Bucket::Pending => State::Waiting,
        }
    }
}
