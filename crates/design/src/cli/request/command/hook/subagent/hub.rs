use std::fmt;
use std::iter;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use design_messages::{CheckName, CommitSha, PullNumber};
use dispatch_ports::{Deadline, Due, Pause, Ran, Timeout};
use grammar::spec_line::Attachment;
use invariants::Ignoring;
use json_ports::{Decode, Json};

use super::super::super::crew::{Asked, Cut, Tape};
use super::boundary::{Branch, Branched, Compared, FailedJob, First, JobId, Opened, Pushed};
use crate::cli::request::ignored::Ignored;
use crate::cli::request::{Landed, Repository, UnreadStep};

#[derive(Clone, Copy, Debug)]
pub struct Hub<'hook> {
    directory: &'hook Path,
    repository: &'hook Repository,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red(Vec<FailedJob>),
}

#[derive(Debug, serde::Deserialize)]
struct Head {
    #[serde(rename = "headRefOid")]
    head: CommitSha,
}

#[derive(Debug, serde::Deserialize)]
struct Merging {
    mergeable: Mergeability,
    #[serde(rename = "mergeStateStatus")]
    state: MergeState,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Mergeability {
    #[serde(rename = "MERGEABLE")]
    Clean,
    #[serde(rename = "CONFLICTING")]
    Conflicting,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum MergeState {
    Behind,
    Dirty,
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merge {
    Current,
    Stale,
}

#[derive(Debug, serde::Deserialize)]
struct Check {
    name: CheckName,
    bucket: Bucket,
    link: Link,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "String")]
struct Link(Option<JobId>);

#[derive(Debug, serde::Deserialize)]
struct Run {
    #[serde(rename = "databaseId")]
    id: NonZeroU64,
    attempt: NonZeroU32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Attempt {
    run: NonZeroU64,
    number: u32,
}

#[derive(Debug, serde::Deserialize)]
struct Attempted {
    jobs: Vec<Job>,
}

#[derive(Debug, serde::Deserialize)]
struct Job {
    name: CheckName,
    conclusion: Conclusion,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Conclusion {
    Success,
    Failure,
    TimedOut,
    Cancelled,
    Skipped,
    Neutral,
    StartupFailure,
    #[serde(other)]
    Unfinished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unopened {
    Unpushed(Option<CommitSha>),
    Unread(UnreadStep),
}

#[derive(Clone, Copy, Debug)]
struct Body<'spec>(&'spec Attachment);

#[derive(Debug, serde::Deserialize)]
struct Found {
    data: Data,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    repository: Holder,
}

#[derive(Debug, serde::Deserialize)]
struct Holder {
    #[serde(rename = "ref")]
    reference: Option<Targeted>,
}

#[derive(Debug, serde::Deserialize)]
struct Targeted {
    target: Oid,
}

#[derive(Debug, serde::Deserialize)]
struct Oid {
    oid: CommitSha,
}

impl<'hook> Hub<'hook> {
    const PROGRAM: &'static str = "gh";
    const API: &'static str = "api";
    const QUICK: Timeout = Timeout::new(60);
    const WATCH: Timeout = Timeout::new(20 * 60);
    const POLL: Pause = Pause::new(10);
    const LANDED: &'static str = "100";
    const UNLOGGED: &'static str = "(the log could not be read)";
    const BASE: &'static str = "main";
    const REMOTE: &'static str = "query($owner:String!,$name:String!,$ref:String!){repository(owner:$owner,name:$name)\
                                  {ref(qualifiedName:$ref){target{oid}}}}";

    pub fn new(directory: &'hook Path, repository: &'hook Repository) -> Hub<'hook> {
        Hub { directory, repository }
    }

    fn ran(&self, tape: &Tape<'_>, arguments: &[&str], timeout: Timeout) -> Result<Option<Ran>, Asked> {
        let named = self.repository.to_string();
        let scoped = (arguments.first() != Some(&Self::API)).then_some(["-R", named.as_str()]);
        let spoken: Vec<&str> = arguments.iter().copied().chain(scoped.into_iter().flatten()).collect();
        tape.spoke(Self::PROGRAM, &spoken, self.directory, timeout)
    }

    fn read<T: Decode, Unread>(&self, tape: &Tape<'_>, arguments: &[&str], unread: Unread) -> Result<T, Cut<Unread>> {
        let ran = self.ran(tape, arguments, Self::QUICK)?;
        match ran.as_ref().and_then(Ran::succeeded).map(|printed| Json::read(printed.as_ref())) {
            Some(Ok(read)) => Ok(read),
            Some(Err(_)) | None => Err(Cut::Failed(unread)),
        }
    }

    pub fn viewed<T: Decode, Unread>(
        &self,
        tape: &Tape<'_>,
        pull: PullNumber,
        fields: &str,
        unread: Unread,
    ) -> Result<T, Cut<Unread>> {
        self.read(tape, &["pr", "view", &pull.to_string(), "--json", fields], unread)
    }

    pub fn landed(&self, tape: &Tape<'_>) -> Result<Option<Vec<Landed>>, Asked> {
        let asked =
            ["pr", "list", "--state", "all", "--limit", Self::LANDED, "--json", "number,state,title,headRefName,body"];
        Ok(Cut::settled(self.read(tape, &asked, UnreadStep::Landed))?.ignoring(Ignored::Undrained))
    }

    pub fn cited(&self, tape: &Tape<'_>, branched: &Branched) -> Result<Attachment, Cut<Unopened>> {
        let held = self.remote(tape, branched.branch())?;
        (held == Some(branched.head())).then_some(()).ok_or(Cut::Failed(Unopened::Unpushed(held)))?;
        let first = self.first(tape, branched.head())?;
        first.spec().cloned().ok_or(Cut::Failed(Unopened::Unread(UnreadStep::Spec)))
    }

    pub fn opened(
        &self,
        tape: &Tape<'_>,
        branched: &Branched,
        spec: &Attachment,
        title: &str,
    ) -> Result<Pushed, Cut<Unopened>> {
        let branch = branched.branch().to_string();
        let body = Body(spec).to_string();
        let unopened = Unopened::Unread(UnreadStep::Open);
        let asked = ["pr", "create", "--head", &branch, "--base", Self::BASE, "--title", title, "--body", &body];
        self.ran(tape, &asked, Self::QUICK)?.as_ref().and_then(Ran::succeeded).ok_or(Cut::Failed(unopened))?;
        let opened: Opened = self.read(tape, &["pr", "view", &branch, "--json", "number"], unopened)?;
        Ok(Pushed::new(opened.pull(), branched.head()))
    }

    fn remote(&self, tape: &Tape<'_>, branch: &Branch) -> Result<Option<CommitSha>, Cut<Unopened>> {
        let reference = format!("ref=refs/heads/{branch}");
        let query = format!("query={}", Self::REMOTE);
        let owner = format!("owner={}", self.repository.owner());
        let name = format!("name={}", self.repository.name());
        let asked = [Self::API, "graphql", "-F", &owner, "-F", &name, "-f", &reference, "-f", &query];
        let found: Found = self.read(tape, &asked, Unopened::Unread(UnreadStep::Remote))?;
        Ok(found.data.repository.reference.map(|targeted| targeted.target.oid))
    }

    fn first(&self, tape: &Tape<'_>, head: CommitSha) -> Result<First, Cut<Unopened>> {
        let compare = format!("repos/{}/compare/{}...{head}", self.repository, Self::BASE);
        let unread = Unopened::Unread(UnreadStep::Commits);
        self.read::<Compared, _>(tape, &[Self::API, &compare], unread)?.first().ok_or(Cut::Failed(unread))
    }

    pub fn head(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<CommitSha, Cut<UnreadStep>> {
        self.viewed::<Head, _>(tape, pull, "headRefOid", UnreadStep::Head).map(|read| read.head)
    }

    pub fn mergeable(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<Merge, Cut<UnreadStep>> {
        let fields = "mergeable,mergeStateStatus";
        self.viewed::<Merging, _>(tape, pull, fields, UnreadStep::Mergeable).map(Merging::merge)
    }

    pub fn lacking(&self, tape: &Tape<'_>, head: CommitSha) -> Result<Vec<String>, Asked> {
        let compare = format!("repos/{}/compare/{head}...{}", self.repository, Self::BASE);
        let read = Cut::settled(self.read::<Compared, _>(tape, &[Self::API, &compare], UnreadStep::Lacking))?;
        Ok(read.ignoring(Ignored::Unlacked).map_or_default(Compared::titles))
    }

    pub fn settled(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<Verdict, Cut<UnreadStep>> {
        let deadline = Deadline::after(tape.now(), Self::WATCH);
        iter::repeat_with(|| tape.now())
            .take_while(|now| deadline.due(*now) == Due::Pending)
            .find_map(|_now| self.polled(tape, pull).transpose())
            .unwrap_or(Err(Cut::Failed(UnreadStep::Overdue)))
    }

    fn polled(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<Option<Verdict>, Cut<UnreadStep>> {
        let listed = self.listed(tape, pull)?;
        let verdict = listed.map_or(Ok(None), |checks| Verdict::read(&checks)).map_err(Cut::Failed)?;
        verdict.map_or_else(
            || {
                self.ran(tape, &["pr", "checks", &pull.to_string(), "--watch", "--fail-fast"], Self::WATCH)?;
                tape.paused(Self::POLL)?;
                Ok(None)
            },
            |found| Ok(Some(found)),
        )
    }

    pub fn requested(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<(), Cut<UnreadStep>> {
        let listed = self.listed(tape, pull)?.unwrap_or_default();
        let number = pull.to_string();
        let label = |change: &str| self.ran(tape, &["pr", "edit", &number, change, Check::GATE], Self::QUICK);
        let added = Check::gate(&listed).is_some() || {
            label("--remove-label")?;
            label("--add-label")?.as_ref().and_then(Ran::succeeded).is_some()
        };
        added.then_some(()).ok_or(Cut::Failed(UnreadStep::Label))
    }

    fn listed(&self, tape: &Tape<'_>, pull: PullNumber) -> Result<Option<Vec<Check>>, Asked> {
        let ran = self.ran(tape, &["pr", "checks", &pull.to_string(), "--json", "name,bucket,link"], Self::QUICK)?;
        let printed = ran.as_ref().and_then(Ran::printed);
        Ok(printed.and_then(|read| Json::read(read.as_ref()).ignoring(Ignored::Unlisted)))
    }

    pub fn gated(&self, tape: &Tape<'_>, head: CommitSha) -> Result<Option<CheckName>, Cut<UnreadStep>> {
        let sha = head.to_string();
        let asked = ["run", "list", "--commit", &sha, "--json", "databaseId,attempt"];
        let runs: Vec<Run> = self.read(tape, &asked, UnreadStep::Runs)?;
        runs.iter()
            .flat_map(Run::earlier)
            .map(|attempt| self.attempted(tape, attempt))
            .find_map(Result::transpose)
            .transpose()
    }

    fn attempted(&self, tape: &Tape<'_>, attempt: Attempt) -> Result<Option<CheckName>, Cut<UnreadStep>> {
        let asked =
            ["run", "view", &attempt.run.to_string(), "--attempt", &attempt.number.to_string(), "--json", "jobs"];
        let attempted: Attempted = self.read(tape, &asked, UnreadStep::Attempt)?;
        Ok(attempted.jobs.into_iter().find_map(Job::gated))
    }

    pub fn log(&self, tape: &Tape<'_>, job: JobId) -> Result<String, Asked> {
        let ran = self.ran(tape, &["run", "view", "--job", &job.to_string(), "--log-failed"], Self::QUICK)?;
        Ok(ran.as_ref().and_then(Ran::succeeded).map_or_else(|| Self::UNLOGGED.to_owned(), ToString::to_string))
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
        let settled = Check::gate(checks).is_some() && checks.iter().all(|check| check.bucket.state() == State::Passed);
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

impl Merging {
    fn merge(self) -> Merge {
        match (self.mergeable, self.state) {
            (Mergeability::Conflicting, MergeState::Behind | MergeState::Dirty | MergeState::Other)
            | (Mergeability::Clean | Mergeability::Unknown, MergeState::Behind | MergeState::Dirty) => Merge::Stale,
            (Mergeability::Clean | Mergeability::Unknown, MergeState::Other) => Merge::Current,
        }
    }
}

impl Run {
    fn earlier(&self) -> impl Iterator<Item = Attempt> + use<'_> {
        (1..self.attempt.get()).map(|number| Attempt { run: self.id, number })
    }
}

impl Job {
    fn gated(self) -> Option<CheckName> {
        match self.conclusion {
            Conclusion::Failure | Conclusion::TimedOut => Some(self.name),
            Conclusion::Success
            | Conclusion::Cancelled
            | Conclusion::Skipped
            | Conclusion::Neutral
            | Conclusion::StartupFailure
            | Conclusion::Unfinished => None,
        }
    }
}

impl Check {
    const GATE: &'static str = "mutants";

    fn gate(checks: &[Check]) -> Option<&Check> {
        checks.iter().find(|check| check.name.to_string() == Check::GATE)
    }

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
