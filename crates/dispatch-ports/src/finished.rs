use std::fmt;
use std::str::FromStr;

use named_derive::Named;
use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Tokens(u64);

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spend {
    #[name("input")]
    Input,
    #[name("output")]
    Output,
    #[name("cache-written")]
    CacheWritten,
    #[name("cache-read")]
    CacheRead,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    input: Tokens,
    #[serde(rename = "output_tokens")]
    output: Tokens,
    #[serde(rename = "cache_creation_input_tokens", default)]
    cache_written: Tokens,
    #[serde(rename = "cache_read_input_tokens", default)]
    cache_read: Tokens,
}

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Reply(String);

#[derive(Debug, thiserror::Error)]
#[error("the agent's reply holds a NUL character")]
pub struct Nul;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finished {
    reply: Reply,
    usage: Usage,
}

#[derive(Debug, thiserror::Error)]
pub enum Unreadable {
    #[error("the agent's output is not its JSON result: {0}")]
    Json(#[source] serde_json::Error),
    #[error("the agent ended without a successful result")]
    Unsuccessful,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Subtype {
    #[serde(rename = "success")]
    Success,
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
struct Envelope {
    subtype: Subtype,
    result: Option<Reply>,
    usage: Usage,
}

impl Spend {
    const EVERY: [Spend; 4] = [Spend::Input, Spend::Output, Spend::CacheWritten, Spend::CacheRead];

    #[must_use]
    pub fn spent_in(self, usage: &Usage) -> Tokens {
        match self {
            Spend::Input => usage.input,
            Spend::Output => usage.output,
            Spend::CacheWritten => usage.cache_written,
            Spend::CacheRead => usage.cache_read,
        }
    }
}

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("usage")?;
        Spend::EVERY.iter().try_for_each(|spend| write!(formatter, " {spend} {}", spend.spent_in(self)))
    }
}

impl Usage {
    #[must_use]
    pub fn new(input: Tokens, output: Tokens, cache_written: Tokens, cache_read: Tokens) -> Usage {
        Usage { input, output, cache_written, cache_read }
    }
}

impl Finished {
    #[must_use]
    pub fn reply(&self) -> &Reply {
        &self.reply
    }

    #[must_use]
    pub fn usage(&self) -> Usage {
        self.usage
    }
}

impl TryFrom<String> for Reply {
    type Error = Nul;

    fn try_from(text: String) -> Result<Reply, Nul> {
        match text.chars().find(|letter| *letter == '\0') {
            Some(_) => Err(Nul),
            None => Ok(Reply(text)),
        }
    }
}

impl AsRef<str> for Reply {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Finished {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Finished, Unreadable> {
        let envelope: Envelope = serde_json::from_str(text).map_err(Unreadable::Json)?;
        envelope.finished().ok_or(Unreadable::Unsuccessful)
    }
}

impl Envelope {
    fn finished(self) -> Option<Finished> {
        let usage = self.usage;
        Some(self.subtype)
            .filter(|subtype| *subtype == Subtype::Success)
            .and(self.result)
            .map(|reply| Finished { reply, usage })
    }
}
