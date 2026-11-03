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

    pub fn lowered(self, measured: Seconds) -> Result<Seconds, Refusal> {
        match self {
            Limit::Levels { .. } => Err(Refusal::Levels),
            Limit::Record(record) if measured > record => Err(Refusal::Above { record }),
            Limit::Record(_) => Ok(measured),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Refusal {
    Levels,
    Above { record: Seconds },
}

impl Refusal {
    pub fn refusing(self, stage: Stage, measured: Seconds) -> Failure<'static> {
        match self {
            Refusal::Levels => Failure::Levels { stage },
            Refusal::Above { record } => Failure::Above { stage, measured, record },
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
