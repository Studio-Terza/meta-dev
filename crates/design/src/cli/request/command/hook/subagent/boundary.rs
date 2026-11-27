use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use design_messages::PullNumber;
use grammar::{AgentReport, Ended, JobLink, Target};
use grammar_lines::Lines;
use grammar_spec_line::Attachment;
use hex::FromHex;
use invariants::Ignoring;
use newtype::{Checked, Display, Newtype};
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

#[derive(Newtype, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked)]
pub struct Branch(String);

#[derive(Newtype, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked)]
pub struct Subject(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    subject: Subject,
    spec: Option<Attachment>,
}

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

#[derive(Debug, thiserror::Error)]
#[error("not a branch name of lowercase words joined by hyphens and slashes")]
pub struct Unbranched;

#[derive(Debug, thiserror::Error)]
#[error("a commit subject is blank or holds a control character")]
pub struct Unsubjected;

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
            Target::Branch(branch) => {
                At::Branch(Branched { branch: Branch::try_new(branch.to_owned()).ignoring(Ignored::Untyped)?, head })
            }
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
    pub fn read(printed: &str) -> Option<First> {
        let lines: Vec<String> = match printed.parse::<Lines>() {
            Ok(lines) => lines.into_iter().collect(),
            Err(unsplit) => match unsplit {},
        };
        let subject = Subject::try_new(lines.first()?.clone()).ignoring(Ignored::Unspecified)?;
        let spec = lines.get(1).and_then(|trailer| Attachment::try_new(trailer.clone()).ignoring(Ignored::Unspecified));
        Some(First { subject, spec })
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    pub fn spec(&self) -> Option<&Attachment> {
        self.spec.as_ref()
    }
}

impl Opened {
    pub fn pull(&self) -> PullNumber {
        PullNumber::new(self.number)
    }
}

impl Checked for Branch {
    type Error = Unbranched;

    fn checked(text: &String) -> Result<&String, Unbranched> {
        match Target::read(text) {
            Some(Target::Branch(_branch)) => Ok(text),
            Some(Target::Pull(_)) | None => Err(Unbranched),
        }
    }
}

impl Checked for Subject {
    type Error = Unsubjected;

    fn checked(text: &String) -> Result<&String, Unsubjected> {
        let said = text.chars().next().is_some() && text.chars().all(|letter| !letter.is_control());
        said.then_some(text).ok_or(Unsubjected)
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
