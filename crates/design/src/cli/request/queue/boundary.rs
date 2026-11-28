use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use grammar::{Cited as Detail, Lines};
use grammar_spec_line::{Attachment, Line};
use invariants::Ignoring;
use serde::{Deserialize, Serialize};

use super::{Closed, Escalated, Item, Kept, Ruling, Target};
use crate::cli::request::ignored::Ignored;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct HeadRef(String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct MessageRef(String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Cited(Attachment);

#[derive(Debug, Deserialize)]
pub(crate) struct Landed {
    number: NonZeroU32,
    state: Settled,
    title: String,
    #[serde(rename = "headRefName")]
    branch: HeadRef,
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
pub(crate) struct Unreferenced;

pub(crate) trait Named {
    fn spec(&self, cited: &Attachment) -> Option<String>;

    fn owner(&self, message: &str) -> Option<String>;
}

#[derive(Debug)]
pub(crate) struct Shown<'shown, N: Named> {
    item: &'shown Item,
    landed: &'shown [Landed],
    named: &'shown N,
}

impl Item {
    pub(crate) fn shown<'shown, N: Named>(
        &'shown self,
        landed: &'shown [Landed],
        named: &'shown N,
    ) -> Shown<'shown, N> {
        Shown { item: self, landed, named }
    }
}

impl Landed {
    pub(super) fn number(&self) -> NonZeroU32 {
        self.number
    }

    pub(super) fn branch(&self) -> &HeadRef {
        &self.branch
    }

    pub(super) fn settled(&self) -> Option<Closed> {
        match self.state {
            Settled::Merged | Settled::Closed => Some(Closed),
            Settled::Open => None,
        }
    }

    pub(super) fn merged(&self, cited: &Cited) -> Option<Closed> {
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

impl<Shown: fmt::Display> From<&Shown> for HeadRef {
    fn from(branch: &Shown) -> HeadRef {
        HeadRef(branch.to_string())
    }
}

impl fmt::Display for HeadRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
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
