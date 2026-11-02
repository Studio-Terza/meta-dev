use std::fmt;

use serde::Deserialize;

use crate::cli::failure::Failure;
use crate::cli::gate::seconds::Seconds;
use crate::cli::gate::verdict::Verdict;
use crate::cli::stage::Stage;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Limit {
    Levels { warn: Seconds, block: Seconds },
    Record(Seconds),
}

impl Limit {
    pub fn ordered(self) -> Option<Limit> {
        match self {
            Limit::Levels { warn, block } if warn > block => None,
            Limit::Levels { .. } | Limit::Record(_) => Some(self),
        }
    }

    pub fn judge(&self, measured: Seconds) -> Verdict {
        match self {
            Limit::Levels { warn, block } => match (measured > *block, measured > *warn) {
                (true, _) => Verdict::Block,
                (false, true) => Verdict::Warn,
                (false, false) => Verdict::Within,
            },
            Limit::Record(record) if measured > *record => Verdict::Block,
            Limit::Record(_) => Verdict::Within,
        }
    }

    pub fn lowered(self, stage: &Stage, measured: Seconds) -> Result<Seconds, Failure> {
        match self {
            Limit::Levels { .. } => Err(Failure::Levels { stage: stage.clone() }),
            Limit::Record(record) if measured > record => {
                Err(Failure::Above { stage: stage.clone(), measured, record })
            }
            Limit::Record(_) => Ok(measured),
        }
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Limit::Levels { warn, block } => write!(formatter, "warn above {warn}, block above {block}"),
            Limit::Record(record) => write!(formatter, "record {record}, lowered only by ci ratchet-lower"),
        }
    }
}
