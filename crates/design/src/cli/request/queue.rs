use std::fmt;
use std::num::NonZeroU32;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use file_ports::Kind;
use grammar::{Cited as Detail, Lines};
use grammar_spec_line::{Attachment, Line};
use invariants::Ignoring;
use serde::{Deserialize, Serialize};

use super::ignored::Ignored;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Queue {
    #[serde(default, rename = "item", skip_serializing_if = "Vec::is_empty")]
    items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Item {
    said: String,
    #[serde(flatten)]
    waits: Waits,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Waits {
    Spec(Cited),
    Pull(NonZeroU32),
    Escalation(Target),
    Worktree(PathBuf),
    Ruling(Ruling),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Pull(NonZeroU32),
    Branch(String),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ruling {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Cited(Attachment);

#[derive(Debug, Deserialize)]
pub struct Landed {
    number: NonZeroU32,
    state: Settled,
    #[serde(rename = "headRefName")]
    branch: String,
    body: String,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Settled {
    #[serde(rename = "MERGED")]
    Merged,
    #[serde(rename = "CLOSED")]
    Closed,
    #[serde(rename = "OPEN")]
    Open,
}

#[derive(Clone, Copy, Debug)]
struct Closed;

impl Queue {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub fn with(self, item: Item) -> Queue {
        Queue { items: self.items.into_iter().chain([item]).collect() }
    }

    #[must_use]
    pub fn drained(self, landed: &[Landed], kind: impl Fn(&Path) -> Kind) -> Queue {
        Queue { items: self.items.into_iter().filter(|item| item.closed(landed, &kind).is_none()).collect() }
    }

    #[must_use]
    pub fn cleared(self, passed: &[Target]) -> Queue {
        let open = |item: &Item| match &item.waits {
            Waits::Escalation(target) => passed.contains(target).not(),
            Waits::Spec(_) | Waits::Pull(_) | Waits::Worktree(_) | Waits::Ruling(_) => true,
        };
        Queue { items: self.items.into_iter().filter(open).collect() }
    }
}

impl Item {
    pub fn new(said: impl fmt::Display, waits: Waits) -> Item {
        Item { said: said.to_string(), waits }
    }

    fn closed(&self, landed: &[Landed], kind: impl Fn(&Path) -> Kind) -> Option<Closed> {
        match &self.waits {
            Waits::Spec(cited) => landed.iter().find(|pull| pull.merged(cited).is_some()).map(|_merged| Closed),
            Waits::Pull(pull) | Waits::Escalation(Target::Pull(pull)) => {
                landed.iter().find(|listed| listed.number == *pull).and_then(Landed::settled)
            }
            Waits::Escalation(Target::Branch(branch)) => {
                landed.iter().find(|listed| listed.branch == *branch).and_then(Landed::settled)
            }
            Waits::Worktree(path) => (kind(path) == Kind::Missing).then_some(Closed),
            Waits::Ruling(Ruling { message }) => message.as_ref().map(|_ruled| Closed),
        }
    }
}

impl Landed {
    fn settled(&self) -> Option<Closed> {
        match self.state {
            Settled::Merged | Settled::Closed => Some(Closed),
            Settled::Open => None,
        }
    }

    fn merged(&self, cited: &Cited) -> Option<Closed> {
        let lines = match self.body.parse::<Lines>() {
            Ok(lines) => lines,
            Err(unsplit) => match unsplit {},
        };
        let first = lines.into_iter().next()?;
        let line = Line::from_str(&first).ignoring(Ignored::Unbodied)?;
        let attached = line.spec()?.attachment()?.to_string();
        let wanted = cited.0.to_string();
        let asked = Detail::each(&wanted);
        let found = Detail::each(&attached).into_iter().find(|landed| asked.contains(landed));
        found.and((self.state == Settled::Merged).then_some(Closed))
    }
}

impl TryFrom<String> for Cited {
    type Error = grammar::Unspelled;

    fn try_from(text: String) -> Result<Cited, grammar::Unspelled> {
        Attachment::try_new(text).map(Cited)
    }
}

impl From<Cited> for String {
    fn from(cited: Cited) -> String {
        cited.0.to_string()
    }
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "- {} (closes on ", self.said)?;
        match &self.waits {
            Waits::Spec(cited) => write!(formatter, "the merge of {}'s pull request", cited.0),
            Waits::Pull(pull) => write!(formatter, "#{pull} merging or closing"),
            Waits::Escalation(Target::Pull(pull)) => write!(formatter, "a passing DONE on #{pull}, or it merging"),
            Waits::Escalation(Target::Branch(branch)) => write!(formatter, "a passing DONE on {branch}"),
            Waits::Worktree(path) => write!(formatter, "{} being removed", path.display()),
            Waits::Ruling(_) => formatter.write_str("an owner ruling's message id recorded as ruling.message"),
        }?;
        formatter.write_str(")")
    }
}
