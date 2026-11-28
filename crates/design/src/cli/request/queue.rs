use std::fmt;
use std::num::NonZeroU32;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use design_messages::CheckName;
use file_ports::Kind;
use grammar_detail::Cited as Detail;
use grammar_lines::Lines;
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
#[serde(rename_all = "kebab-case")]
pub enum Item {
    Spec(Cited),
    Pull(NonZeroU32),
    Escalation(Escalated),
    Worktree(Kept),
    Ruling(Ruling),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Escalated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    at: Option<Target>,
    cause: Cause,
    from: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Pull(NonZeroU32),
    Branch(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Cause {
    Blocked,
    RepeatFailure(CheckName),
    Unread(UnreadStep),
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum UnreadStep {
    Ending,
    Head,
    Mergeable,
    Overdue,
    Checks,
    NoRepository,
    NoSpecs,
    Remote,
    Commits,
    Spec,
    Detail,
    Open,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Kept {
    path: PathBuf,
    held: Held,
    merged: NonZeroU32,
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Held {
    NoRepository,
    Unviewed,
    Unlisted,
    Unstatused,
    Dirty,
    Unpushed,
    Unremoved,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Ruling {
    asked: MessageRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ruled: Option<MessageRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct MessageRef(String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Cited(Attachment);

#[derive(Debug, Deserialize)]
pub struct Landed {
    number: NonZeroU32,
    state: Settled,
    title: String,
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

#[derive(Debug, thiserror::Error)]
#[error("a message id is eight or more hexadecimal digits")]
pub struct Unreferenced;

pub trait Named {
    fn spec(&self, cited: &Attachment) -> Option<String>;

    fn owner(&self, message: &str) -> Option<String>;
}

#[derive(Debug)]
pub struct Shown<'shown, N: Named> {
    item: &'shown Item,
    landed: &'shown [Landed],
    named: &'shown N,
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
        let open = |item: &Item| match item {
            Item::Escalation(Escalated { at: Some(target), .. }) => passed.contains(target).not(),
            Item::Escalation(Escalated { at: None, .. })
            | Item::Spec(_)
            | Item::Pull(_)
            | Item::Worktree(_)
            | Item::Ruling(_) => true,
        };
        Queue { items: self.items.into_iter().filter(open).collect() }
    }
}

impl Item {
    pub fn escalated(at: Option<Target>, cause: Cause, from: &Path) -> Item {
        Item::Escalation(Escalated { at, cause, from: from.to_path_buf() })
    }

    pub fn kept(path: PathBuf, held: Held, merged: NonZeroU32) -> Item {
        Item::Worktree(Kept { path, held, merged })
    }

    pub fn shown<'shown, N: Named>(&'shown self, landed: &'shown [Landed], named: &'shown N) -> Shown<'shown, N> {
        Shown { item: self, landed, named }
    }

    fn closed(&self, landed: &[Landed], kind: impl Fn(&Path) -> Kind) -> Option<Closed> {
        match self {
            Item::Spec(cited) => landed.iter().find(|pull| pull.merged(cited).is_some()).map(|_merged| Closed),
            Item::Pull(pull) | Item::Escalation(Escalated { at: Some(Target::Pull(pull)), .. }) => {
                landed.iter().find(|listed| listed.number == *pull).and_then(Landed::settled)
            }
            Item::Escalation(Escalated { at: Some(Target::Branch(branch)), .. }) => {
                landed.iter().find(|listed| listed.branch == *branch).and_then(Landed::settled)
            }
            Item::Escalation(Escalated { at: None, from: path, .. }) | Item::Worktree(Kept { path, .. }) => {
                (kind(path) == Kind::Missing).then_some(Closed)
            }
            Item::Ruling(Ruling { ruled, .. }) => ruled.as_ref().map(|_ruled| Closed),
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

    fn titled(landed: &[Landed], pull: NonZeroU32) -> String {
        let title = landed.iter().find(|listed| listed.number == pull).map(|listed| format!(" {}", listed.title));
        format!("#{pull}{}", title.unwrap_or_default())
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

impl MessageRef {
    const SHORTEST: usize = 8;
}

impl TryFrom<String> for MessageRef {
    type Error = Unreferenced;

    fn try_from(text: String) -> Result<MessageRef, Unreferenced> {
        let hexadecimal = text.chars().all(|letter| letter.is_ascii_hexdigit());
        (hexadecimal && text.len() >= MessageRef::SHORTEST).then_some(MessageRef(text)).ok_or(Unreferenced)
    }
}

impl From<MessageRef> for String {
    fn from(message: MessageRef) -> String {
        message.0
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cause::Blocked => formatter.write_str("blocked"),
            Cause::RepeatFailure(check) => write!(formatter, "repeat failure of {check}"),
            Cause::Unread(step) => write!(formatter, "unread {step}"),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Pull(pull) => write!(formatter, "#{pull}"),
            Target::Branch(branch) => formatter.write_str(branch),
        }
    }
}

impl<N: Named> fmt::Display for Shown<'_, N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Shown { item, landed, named } = self;
        match item {
            Item::Spec(cited) => {
                let line = named.spec(&cited.0).unwrap_or_else(|| cited.0.to_string());
                write!(formatter, "- {} {line} (closes when its pull request merges)", cited.0)
            }
            Item::Pull(pull) => {
                write!(formatter, "- {} (closes when it merges or closes)", Landed::titled(landed, *pull))
            }
            Item::Escalation(Escalated { at, cause, from }) => {
                write!(formatter, "- agent outcome: {cause}")?;
                match at {
                    Some(Target::Pull(pull)) => write!(formatter, " on {}", Landed::titled(landed, *pull)),
                    Some(Target::Branch(branch)) => write!(formatter, " on {branch}"),
                    None => Ok(()),
                }?;
                let closes = at.as_ref().map_or_else(
                    || format!("{} is removed", from.display()),
                    |target| format!("a DONE on {target} passes, or it merges or closes"),
                );
                write!(formatter, ", from {} (closes when {closes})", from.display())
            }
            Item::Worktree(Kept { path, held, merged }) => write!(
                formatter,
                "- worktree {} kept ({held}) after {} merged (closes when it is removed)",
                path.display(),
                Landed::titled(landed, *merged)
            ),
            Item::Ruling(Ruling { asked, .. }) => {
                let said = named.owner(&asked.0).map_or_else(String::new, |text| format!(": \"{text}\""));
                write!(
                    formatter,
                    "- owner ruling asked in {}{said} (closes when ruling.ruled names the answer)",
                    asked.0
                )
            }
        }
    }
}
