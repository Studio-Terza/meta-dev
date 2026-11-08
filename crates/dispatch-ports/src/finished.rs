mod boundary;

use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

pub use boundary::{Output, Unreadable};

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Tokens(u32);

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

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Reply(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finished {
    reply: Reply,
    usage: Usage,
}

impl Usage {
    const SPENDS: usize = 4;

    #[must_use]
    pub fn new(input: Tokens, output: Tokens, cache_written: Tokens, cache_read: Tokens) -> Usage {
        Usage { input, output, cache_written, cache_read }
    }

    #[must_use]
    pub fn spent(&self) -> [Tokens; Usage::SPENDS] {
        [self.input, self.output, self.cache_written, self.cache_read]
    }
}

impl Finished {
    #[must_use]
    pub fn new(reply: Reply, usage: Usage) -> Finished {
        Finished { reply, usage }
    }

    #[must_use]
    pub fn reply(&self) -> &Reply {
        &self.reply
    }

    #[must_use]
    pub fn usage(&self) -> Usage {
        self.usage
    }
}
