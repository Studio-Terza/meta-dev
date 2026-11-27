use std::fmt;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use design_messages::PullNumber;
use grammar::{AgentReport, Ended, JobLink};
use hex::FromHex;
use invariants::Ignoring;
use newtype::{Display, Newtype};
use serde::Deserialize;

use grammar_lines::Lines;

use super::FailedJob;
use crate::cli::request::ignored::Ignored;

#[derive(Debug)]
pub(crate) struct Excerpt {
    failed: FailedJob,
    lines: Vec<String>,
}

impl Excerpt {
    const LINES: usize = 40;

    pub(crate) fn tailed(failed: FailedJob, log: &str) -> Excerpt {
        let whole: Vec<String> = match log.parse::<Lines>() {
            Ok(lines) => lines.into_iter().collect(),
            Err(unsplit) => match unsplit {},
        };
        let skipped = whole.len().saturating_sub(Excerpt::LINES);
        Excerpt { failed, lines: whole.into_iter().skip(skipped).collect() }
    }
}

impl fmt::Display for Excerpt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Excerpt { failed: FailedJob { check, job }, lines } = self;
        write!(formatter, "\n{check} (job {job}):\n")?;
        lines.iter().try_for_each(|line| writeln!(formatter, "{line}"))
    }
}

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
pub(crate) enum Reminder {
    #[default]
    First,
    Repeated,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(from = "Option<String>")]
pub enum Said {
    Typed(Report),
    #[default]
    Untyped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    Done(Pushed),
    Blocked(Pushed),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pushed {
    pull: PullNumber,
    head: CommitSha,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub(crate) struct CommitSha([u8; 20]);

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobId(NonZeroU64);

#[derive(Debug, thiserror::Error)]
#[error("not a full commit sha of forty hexadecimal digits")]
pub(crate) struct Unhexed;

impl Finishing {
    pub fn said(&self) -> &Said {
        &self.said
    }

    pub(crate) fn reminder(&self) -> Reminder {
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
        let pushed = Pushed { pull: PullNumber::new(read.pull()), head };
        match read.ended() {
            Ended::Done => Some(Report::Done(pushed)),
            Ended::Blocked => Some(Report::Blocked(pushed)),
        }
    }

    pub(crate) fn pushed(self) -> Pushed {
        match self {
            Report::Done(pushed) | Report::Blocked(pushed) => pushed,
        }
    }
}

impl Pushed {
    pub(crate) fn pull(self) -> PullNumber {
        self.pull
    }

    pub(crate) fn head(self) -> CommitSha {
        self.head
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
    pub(crate) fn linked(link: &str) -> Option<JobId> {
        JobLink::read(link).map(|linked| JobId::new(linked.job()))
    }
}
