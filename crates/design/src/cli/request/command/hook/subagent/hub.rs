use std::fmt;
use std::iter;
use std::ops::Not;
use std::path::Path;

use design_messages::{CheckName, PullNumber};
use dispatch_ports::{Clock, Deadline, Due, Pause, Ran, Shell, Timeout};
use file_ports::Files;
use grammar_spec_line::Attachment;
use invariants::Ignoring;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use store_ports::Store;

use super::super::super::Ground;
use super::super::super::crew::Crew;
use super::boundary::{Branch, Branched, CommitSha, Compared, First, JobId, Opened, Pushed};
use super::{FailedJob, UnreadStep};
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Landed, Local, Repository};

#[derive(Debug)]
pub struct Hub<'crew, C: Clock, G: Shell> {
    crew: &'crew Crew<'crew, C, G>,
    directory: &'crew Path,
    repository: &'crew Repository,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red(Vec<FailedJob>),
}

#[derive(Debug, Deserialize)]
struct Head {
    #[serde(rename = "headRefOid")]
    head: CommitSha,
}

#[derive(Debug, Deserialize)]
struct Merging {
    mergeable: Mergeability,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mergeability {
    #[serde(rename = "MERGEABLE")]
    Clean,
    #[serde(rename = "CONFLICTING")]
    Conflicting,
    #[serde(rename = "UNKNOWN")]
    Unknown,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unopened {
    Unpushed(Option<CommitSha>),
    Unread(UnreadStep),
}

#[derive(Clone, Copy, Debug)]
struct Body<'spec>(&'spec Attachment);

#[derive(Debug, Deserialize)]
struct Found {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    repository: Holder,
}

#[derive(Debug, Deserialize)]
struct Holder {
    #[serde(rename = "ref")]
    reference: Option<Targeted>,
}

#[derive(Debug, Deserialize)]
struct Targeted {
    target: Oid,
}

#[derive(Debug, Deserialize)]
struct Oid {
    oid: CommitSha,
}

impl<'crew, C: Clock, G: Shell> Hub<'crew, C, G> {
    const PROGRAM: &'static str = "gh";
    const QUICK: Timeout = Timeout::new(60);
    const WATCH: Timeout = Timeout::new(20 * 60);
    const POLL: Pause = Pause::new(10);
    const LANDED: &'static str = "100";
    const UNLOGGED: &'static str = "(the log could not be read)";
    const BASE: &'static str = "main";
    const REMOTE: &'static str = "query($owner:String!,$name:String!,$ref:String!){repository(owner:$owner,name:$name)\
                                  {ref(qualifiedName:$ref){target{oid}}}}";

    pub fn new(
        crew: &'crew Crew<'crew, C, G>,
        directory: &'crew Path,
        repository: &'crew Repository,
    ) -> Hub<'crew, C, G> {
        Hub { crew, directory, repository }
    }

    fn ran(&self, arguments: &[&str], timeout: Timeout) -> Option<Ran> {
        let named = self.repository.to_string();
        self.spoke(&[arguments, &["-R", &named]].concat(), timeout)
    }

    fn spoke(&self, arguments: &[&str], timeout: Timeout) -> Option<Ran> {
        self.crew.spoke(Hub::<C, G>::PROGRAM, arguments, self.directory, timeout)
    }

    pub fn viewed<T: DeserializeOwned>(&self, pull: PullNumber, fields: &str) -> Option<T> {
        let ran = self.ran(&["pr", "view", &pull.to_string(), "--json", fields], Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::succeeded)?;
        serde_json::from_str(printed.as_ref()).ignoring(Ignored::Unviewed)
    }

    pub fn landed(&self) -> Option<Vec<Landed>> {
        let asked =
            ["pr", "list", "--state", "all", "--limit", Self::LANDED, "--json", "number,state,title,headRefName,body"];
        let ran = self.ran(&asked, Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::succeeded)?;
        serde_json::from_str(printed.as_ref()).ignoring(Ignored::Undrained)
    }

    pub fn opened<F: Files, S: Store>(
        &self,
        branched: &Branched,
        ground: &Ground<'_, F, S>,
        local: &Local,
    ) -> Result<Pushed, Unopened> {
        let held = self.remote(branched.branch())?;
        (held == Some(branched.head())).then_some(()).ok_or(Unopened::Unpushed(held))?;
        let first = self.first(branched.head())?;
        let spec = first.spec().ok_or(UnreadStep::Spec)?;
        let specs = local.specs().ok_or(UnreadStep::NoSpecs)?;
        let title = ground.titled(specs, spec).ok_or(UnreadStep::Detail)?.to_string();
        let branch = branched.branch().to_string();
        let body = Body(spec).to_string();
        let asked =
            ["pr", "create", "--head", &branch, "--base", Hub::<C, G>::BASE, "--title", &title, "--body", &body];
        self.ran(&asked, Self::QUICK).as_ref().and_then(Ran::succeeded).ok_or(UnreadStep::Open)?;
        let viewed = self.ran(&["pr", "view", &branch, "--json", "number"], Self::QUICK);
        let printed = viewed.as_ref().and_then(Ran::succeeded).ok_or(UnreadStep::Open)?;
        let opened = serde_json::from_str::<Opened>(printed.as_ref()).map_err(|_unread| UnreadStep::Open)?;
        Ok(Pushed::new(opened.pull(), branched.head()))
    }

    fn remote(&self, branch: &Branch) -> Result<Option<CommitSha>, UnreadStep> {
        let reference = format!("ref=refs/heads/{branch}");
        let query = format!("query={}", Hub::<C, G>::REMOTE);
        let owner = format!("owner={}", self.repository.owner());
        let name = format!("name={}", self.repository.name());
        let asked = ["api", "graphql", "-F", &owner, "-F", &name, "-f", &reference, "-f", &query];
        let ran = self.spoke(&asked, Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(UnreadStep::Remote)?;
        let found: Found = serde_json::from_str(printed.as_ref()).map_err(|_unread| UnreadStep::Remote)?;
        Ok(found.data.repository.reference.map(|targeted| targeted.target.oid))
    }

    fn first(&self, head: CommitSha) -> Result<First, UnreadStep> {
        let compare = format!("repos/{}/compare/{}...{head}", self.repository, Hub::<C, G>::BASE);
        let ran = self.spoke(&["api", &compare], Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::succeeded).ok_or(UnreadStep::Commits)?;
        let compared: Compared = serde_json::from_str(printed.as_ref()).map_err(|_unread| UnreadStep::Commits)?;
        compared.first().ok_or(UnreadStep::Commits)
    }

    pub fn head(&self, pull: PullNumber) -> Result<CommitSha, UnreadStep> {
        self.viewed::<Head>(pull, "headRefOid").map(|read| read.head).ok_or(UnreadStep::Head)
    }

    pub fn mergeable(&self, pull: PullNumber) -> Result<Mergeability, UnreadStep> {
        self.viewed::<Merging>(pull, "mergeable").map(|read| read.mergeable).ok_or(UnreadStep::Mergeable)
    }

    pub fn settled(&self, pull: PullNumber) -> Result<Verdict, UnreadStep> {
        let deadline = Deadline::after(self.crew.now(), Self::WATCH);
        iter::repeat_with(|| self.crew.now())
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .find_map(|_now| self.checks(pull).map(|read| read.or_else(|| self.watched(pull))).transpose())
            .unwrap_or(Err(UnreadStep::Overdue))
    }

    fn watched(&self, pull: PullNumber) -> Option<Verdict> {
        self.ran(&["pr", "checks", &pull.to_string(), "--watch", "--fail-fast"], Self::WATCH);
        self.crew.paused(Self::POLL);
        None
    }

    fn checks(&self, pull: PullNumber) -> Result<Option<Verdict>, UnreadStep> {
        let ran = self.ran(&["pr", "checks", &pull.to_string(), "--json", "name,bucket,link"], Self::QUICK);
        let printed = ran.as_ref().and_then(Ran::printed);
        let listed: Option<Vec<Check>> =
            printed.and_then(|read| serde_json::from_str(read.as_ref()).ignoring(Ignored::Unlisted));
        listed.map_or(Ok(None), |checks| Verdict::read(&checks))
    }

    pub fn log(&self, job: JobId) -> String {
        let ran = self.ran(&["run", "view", "--job", &job.to_string(), "--log-failed"], Self::QUICK);
        ran.as_ref().and_then(Ran::succeeded).map_or_else(|| Hub::<C, G>::UNLOGGED.to_owned(), ToString::to_string)
    }
}

impl Verdict {
    fn read(checks: &[Check]) -> Result<Option<Verdict>, UnreadStep> {
        let failed = checks
            .iter()
            .filter(|check| check.bucket.state() == State::Failed)
            .map(Check::failed)
            .collect::<Option<Vec<FailedJob>>>()
            .ok_or(UnreadStep::Checks)?;
        let settled = checks.is_empty().not() && checks.iter().all(|check| check.bucket.state() == State::Passed);
        Ok(match failed.first() {
            Some(_first) => Some(Verdict::Red(failed)),
            None => settled.then_some(Verdict::Green),
        })
    }

    pub fn failed(self) -> Vec<FailedJob> {
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

impl From<String> for Link {
    fn from(link: String) -> Link {
        Link(JobId::linked(&link))
    }
}

impl From<UnreadStep> for Unopened {
    fn from(step: UnreadStep) -> Unopened {
        Unopened::Unread(step)
    }
}

impl fmt::Display for Body<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "spec: {}\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)", self.0)
    }
}
