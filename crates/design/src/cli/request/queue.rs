mod boundary;

use std::fmt;
use std::num::NonZeroU32;
use std::ops::Not;
use std::path::{Path, PathBuf};

use design_messages::CheckName;
use file_ports::Kind;
use serde::{Deserialize, Serialize};

use boundary::{Cited, Closed, MessageRef};
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
    Uninstalled(Uninstalled),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Uninstalled {
    merged: NonZeroU32,
    step: InstallStep,
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum InstallStep {
    Fetch,
    Checkout,
    Build,
    Unchecked,
    Print,
    Settings,
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
    GatedRerun(CheckName),
    Unread(UnreadStep),
}

#[derive(strum::Display, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum UnreadStep {
    Ending,
    Head,
    Mergeable,
    Label,
    Overdue,
    Checks,
    NoRepository,
    NoSpecs,
    Remote,
    Commits,
    Spec,
    Detail,
    Open,
    Runs,
    Attempt,
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

#[derive(Debug)]
pub(crate) struct Shown<'shown, N: Named> {
    item: &'shown Item,
    landed: &'shown [Landed],
    named: &'shown N,
}

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
            | Item::Ruling(_)
            | Item::Uninstalled(_) => true,
        };
        Queue { items: self.items.into_iter().filter(open).collect() }
    }

    #[must_use]
    pub(crate) fn installed(self) -> Queue {
        Queue { items: self.items.into_iter().filter(|item| matches!(item, Item::Uninstalled(_)).not()).collect() }
    }
}

impl Item {
    pub(crate) fn escalated(at: Option<Target>, cause: Cause, from: &Path) -> Item {
        Item::Escalation(Escalated { at, cause, from: from.to_path_buf() })
    }

    pub(crate) fn kept(path: PathBuf, held: Held, merged: NonZeroU32) -> Item {
        Item::Worktree(Kept { path, held, merged })
    }

    pub(crate) fn uninstalled(merged: NonZeroU32, step: InstallStep) -> Item {
        Item::Uninstalled(Uninstalled { merged, step })
    }

    pub(crate) fn shown<'shown, N: Named>(
        &'shown self,
        landed: &'shown [Landed],
        named: &'shown N,
    ) -> Shown<'shown, N> {
        Shown { item: self, landed, named }
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
            Item::Uninstalled(_) => None,
        }
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cause::Blocked => formatter.write_str("blocked"),
            Cause::RepeatFailure(check) => write!(formatter, "repeat failure of {check}"),
            Cause::GatedRerun(check) => write!(formatter, "green only on a rerun after {check} failed a gate"),
            Cause::Unread(step) => write!(formatter, "unread {step}"),
        }
    }
}

impl<N: Named> fmt::Display for Shown<'_, N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Shown { item, landed, named } = self;
        match item {
            Item::Spec(cited) => {
                let attachment = cited.attachment();
                let line = named.spec(attachment).unwrap_or_else(|| attachment.to_string());
                write!(formatter, "- {attachment} {line} (closes when its pull request merges)")
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
                let said = named.owner(asked.as_ref()).map_or_else(String::new, |text| format!(": \"{text}\""));
                write!(formatter, "- owner ruling asked in {asked}{said} (closes when ruling.ruled names the answer)")
            }
            Item::Uninstalled(Uninstalled { merged, step }) => write!(
                formatter,
                "- design not installed after {} merged: the {step} step failed (closes when a later merge installs)",
                Landed::titled(landed, *merged)
            ),
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
