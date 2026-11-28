use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use grammar_detail::Cited as Detail;
use grammar_lines::Lines;
use grammar_spec_line::{Attachment, Line};
use invariants::Ignoring;
use serde::{Deserialize, Serialize};

use crate::cli::request::ignored::Ignored;

#[derive(Clone, Copy, Debug)]
pub(super) struct Closed;

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

    pub(super) fn titled(landed: &[Landed], pull: NonZeroU32) -> String {
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

impl Cited {
    pub(super) fn attachment(&self) -> &Attachment {
        &self.0
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

impl AsRef<str> for MessageRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
