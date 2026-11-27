use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use design_messages::PullNumber;
use grammar::{AgentReport, BranchName, Ended, JobLink, RepositoryName, SpecTrailer, Target};
use grammar_spec_line::Attachment;
use hex::FromHex;
use invariants::Ignoring;
use newtype::{Display, Newtype};
use serde::Deserialize;

use crate::cli::request::ignored::Ignored;

#[derive(Debug, Deserialize)]
pub struct Finishing {
    #[serde(rename = "stop_hook_active", default)]
    reminder: Reminder,
    #[serde(rename = "last_assistant_message", default)]
    said: Said,
    cwd: PathBuf,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(from = "bool")]
pub enum Reminder {
    #[default]
    First,
    Repeated,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(from = "Option<String>")]
pub enum Said {
    Typed(Report),
    #[default]
    Untyped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    Done(At),
    Blocked(At),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum At {
    Pull(Pushed),
    Branch(Branched),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pushed {
    pull: PullNumber,
    head: CommitSha,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branched {
    branch: Branch,
    head: CommitSha,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch(String);

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(from = "String")]
pub struct First {
    spec: Option<Attachment>,
}

#[derive(Debug, Deserialize)]
pub struct Compared {
    commits: Vec<Committed>,
}

#[derive(Debug, Deserialize)]
struct Committed {
    commit: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    message: First,
}

#[derive(Debug, Deserialize)]
pub struct Local {
    repository: Repository,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Repository {
    owner: String,
    name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("not a repository of the form owner/name")]
pub struct Unrepositoried;

#[derive(Debug, Deserialize)]
pub struct Opened {
    number: NonZeroU32,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct CommitSha([u8; 20]);

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobId(NonZeroU64);

#[derive(Debug, thiserror::Error)]
#[error("not a full commit sha of forty hexadecimal digits")]
pub struct Unhexed;

impl Finishing {
    pub fn said(&self) -> &Said {
        &self.said
    }

    pub fn reminder(&self) -> Reminder {
        self.reminder
    }

    pub fn directory(&self) -> &Path {
        &self.cwd
    }
}

impl From<bool> for Reminder {
    fn from(active: bool) -> Reminder {
        if active { Reminder::Repeated } else { Reminder::First }
    }
}

impl From<Option<String>> for Said {
    fn from(said: Option<String>) -> Said {
        said.as_deref().and_then(AgentReport::read).and_then(Report::typed).map_or_default(Said::Typed)
    }
}

impl Report {
    fn typed(read: AgentReport<'_>) -> Option<Report> {
        let head = read.head().parse::<CommitSha>().ignoring(Ignored::Untyped)?;
        let at = match read.target() {
            Target::Pull(pull) => At::Pull(Pushed::new(PullNumber::new(pull), head)),
            Target::Branch(branch) => At::Branch(Branched { branch: Branch::from(branch), head }),
        };
        match read.ended() {
            Ended::Done => Some(Report::Done(at)),
            Ended::Blocked => Some(Report::Blocked(at)),
        }
    }
}

impl Pushed {
    pub fn new(pull: PullNumber, head: CommitSha) -> Pushed {
        Pushed { pull, head }
    }

    pub fn pull(self) -> PullNumber {
        self.pull
    }

    pub fn head(self) -> CommitSha {
        self.head
    }
}

impl Branched {
    pub fn branch(&self) -> &Branch {
        &self.branch
    }

    pub fn head(&self) -> CommitSha {
        self.head
    }
}

impl First {
    pub fn spec(&self) -> Option<&Attachment> {
        self.spec.as_ref()
    }
}

impl From<String> for First {
    fn from(message: String) -> First {
        let trailer = SpecTrailer::read(&message).map(|trailer| trailer.value().to_owned());
        First { spec: trailer.and_then(|value| Attachment::try_new(value).ignoring(Ignored::Unspecified)) }
    }
}

impl Compared {
    pub fn first(self) -> Option<First> {
        self.commits.into_iter().next().map(|committed| committed.commit.message)
    }
}

impl Local {
    pub fn repository(self) -> Repository {
        self.repository
    }
}

impl Repository {
    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<String> for Repository {
    type Error = Unrepositoried;

    fn try_from(text: String) -> Result<Repository, Unrepositoried> {
        let read = RepositoryName::read(&text).ok_or(Unrepositoried)?;
        Ok(Repository { owner: read.owner().to_owned(), name: read.name().to_owned() })
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
    }
}

impl Opened {
    pub fn pull(&self) -> PullNumber {
        PullNumber::new(self.number)
    }
}

impl From<BranchName<'_>> for Branch {
    fn from(lexed: BranchName<'_>) -> Branch {
        Branch(lexed.to_string())
    }
}

impl fmt::Display for Branch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for CommitSha {
    type Err = Unhexed;

    fn from_str(text: &str) -> Result<CommitSha, Unhexed> {
        <[u8; 20]>::from_hex(text).map(CommitSha).map_err(|_unhexed| Unhexed)
    }
}

impl TryFrom<String> for CommitSha {
    type Error = Unhexed;

    fn try_from(text: String) -> Result<CommitSha, Unhexed> {
        text.parse()
    }
}

impl fmt::Display for CommitSha {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0))
    }
}

impl JobId {
    pub fn linked(link: &str) -> Option<JobId> {
        JobLink::read(link).map(|linked| JobId::new(linked.job()))
    }
}
