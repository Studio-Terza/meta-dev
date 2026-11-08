use std::str::FromStr;

use grammar::reply::Inked;
use serde::{Deserialize, Deserializer, de};

use super::{Finished, Reply, Told, Usage};
use crate::{Stderr, Watched};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unreadable {
    #[error("the agent's output is not its JSON result: {failure}; {stderr}")]
    Json { failure: String, stderr: Stderr },
    #[error("the agent ended without a successful result; {stderr}")]
    Unsuccessful { stderr: Stderr },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{told}")]
pub struct Errored {
    told: Told,
    usage: Usage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Finished(Finished),
    Errored(Errored),
    Unreadable(Unreadable),
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} holds no ink or holds a NUL")]
pub struct Uninked(String);

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Subtype {
    #[serde(rename = "success")]
    Success,
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Erred;

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(from = "bool")]
struct Verdict(Option<Erred>);

#[derive(Deserialize, Debug)]
struct Envelope {
    subtype: Subtype,
    #[serde(rename = "is_error", default)]
    verdict: Verdict,
    result: Option<Told>,
    usage: Usage,
}

impl Errored {
    #[must_use]
    pub fn new(told: Told, usage: Usage) -> Errored {
        Errored { told, usage }
    }

    #[must_use]
    pub fn usage(&self) -> Usage {
        self.usage
    }
}

impl From<bool> for Verdict {
    fn from(errored: bool) -> Verdict {
        Verdict(errored.then_some(Erred))
    }
}

impl Envelope {
    fn output(self) -> Option<Output> {
        match self.verdict.0 {
            Some(Erred) => self.result.map(|told| Output::Errored(Errored { told, usage: self.usage })),
            None => Some(self.subtype)
                .filter(|subtype| *subtype == Subtype::Success)
                .and(self.result)
                .map(|told| Output::Finished(Finished { reply: Reply::from(told), usage: self.usage })),
        }
    }
}

impl Output {
    #[must_use]
    pub fn read(text: &str, stderr: Stderr) -> Output {
        match serde_json::from_str::<Envelope>(text) {
            Ok(envelope) => envelope.output().unwrap_or(Output::Unreadable(Unreadable::Unsuccessful { stderr })),
            Err(failure) => Output::Unreadable(Unreadable::Json { failure: failure.to_string(), stderr }),
        }
    }
}

impl From<Output> for Watched {
    fn from(output: Output) -> Watched {
        match output {
            Output::Finished(finished) => Watched::Finished(finished),
            Output::Errored(errored) => Watched::Errored(errored),
            Output::Unreadable(unreadable) => Watched::Unreadable(unreadable),
        }
    }
}

impl FromStr for Told {
    type Err = Uninked;

    fn from_str(text: &str) -> Result<Told, Uninked> {
        Inked::read(text).map(|inked| Told::new(inked.as_ref().to_owned())).ok_or_else(|| Uninked(text.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Told {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Told, D::Error> {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}
