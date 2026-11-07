use std::str::FromStr;

use serde::Deserialize;

use super::{Finished, Reply, Usage};

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

impl FromStr for Finished {
    type Err = Unreadable;

    fn from_str(text: &str) -> Result<Finished, Unreadable> {
        let envelope: Envelope = serde_json::from_str(text).map_err(Unreadable::Json)?;
        let usage = envelope.usage;
        Some(envelope.subtype)
            .filter(|subtype| *subtype == Subtype::Success)
            .and(envelope.result)
            .map(|reply| Finished { reply, usage })
            .ok_or(Unreadable::Unsuccessful)
    }
}
