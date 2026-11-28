mod boundary;

use std::fmt;
use std::num::NonZeroU32;
use std::ops::Not;
use std::path::{Path, PathBuf};

use design_messages::CheckName;
use file_ports::Kind;
use serde::{Deserialize, Serialize};

use boundary::{Cited, MessageRef};
pub(crate) use boundary::{HeadRef, Landed, Named};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Queue {
    #[serde(default, rename = "item", skip_serializing_if = "Vec::is_empty")]
    items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Item {
    Spec(Cited),
    Pull(NonZeroU32),
    Escalation(Escalated),
    Worktree(Kept),
    Ruling(Ruling),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Escalated {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    at: Option<Target>,
    cause: Cause,
    from: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Target {
    Pull(NonZeroU32),
    Branch(HeadRef),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Cause {
    Blocked,
    RepeatFailure(CheckName),
    Unread(UnreadStep),
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum UnreadStep {
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
pub(crate) struct Kept {
    path: PathBuf,
    held: Held,
    merged: NonZeroU32,
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Held {
    NoRepository,
    Unviewed,
    Unlisted,
    Unstatused,
    Dirty,
    Unpushed,
    Unremoved,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Ruling {
    asked: MessageRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ruled: Option<MessageRef>,
}

#[derive(Clone, Copy, Debug)]
struct Closed;

impl Queue {
    pub(crate) fn items(&self) -> &[Item] {
        &self.items
    }

    #[must_use]
    pub(crate) fn with(self, item: Item) -> Queue {
        Queue { items: self.items.into_iter().chain([item]).collect() }
    }

    #[must_use]
    pub(crate) fn drained(self, landed: &[Landed], kind: impl Fn(&Path) -> Kind) -> Queue {
        Queue { items: self.items.into_iter().filter(|item| item.closed(landed, &kind).is_none()).collect() }
    }

    #[must_use]
    pub(crate) fn cleared(self, passed: &[Target]) -> Queue {
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
    pub(crate) fn escalated(at: Option<Target>, cause: Cause, from: &Path) -> Item {
        Item::Escalation(Escalated { at, cause, from: from.to_path_buf() })
    }

    pub(crate) fn kept(path: PathBuf, held: Held, merged: NonZeroU32) -> Item {
        Item::Worktree(Kept { path, held, merged })
    }

    fn closed(&self, landed: &[Landed], kind: impl Fn(&Path) -> Kind) -> Option<Closed> {
        match self {
            Item::Spec(cited) => landed.iter().find(|pull| pull.merged(cited).is_some()).map(|_merged| Closed),
            Item::Pull(pull) | Item::Escalation(Escalated { at: Some(Target::Pull(pull)), .. }) => {
                landed.iter().find(|listed| listed.number() == *pull).and_then(Landed::settled)
            }
            Item::Escalation(Escalated { at: Some(Target::Branch(branch)), .. }) => {
                landed.iter().find(|listed| listed.branch() == branch).and_then(Landed::settled)
            }
            Item::Escalation(Escalated { at: None, from: path, .. }) | Item::Worktree(Kept { path, .. }) => {
                (kind(path) == Kind::Missing).then_some(Closed)
            }
            Item::Ruling(Ruling { ruled, .. }) => ruled.as_ref().map(|_ruled| Closed),
        }
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
            Target::Branch(branch) => write!(formatter, "{branch}"),
        }
    }
}
