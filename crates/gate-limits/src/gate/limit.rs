use std::fmt;

use serde::Deserialize;

use crate::failure::Failure;
use crate::gate::verdict::Verdict;
use crate::seconds::Seconds;
use crate::stage::Stage;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Limit {
    Levels { warn: Seconds, block: Seconds },
    Record(Seconds),
}

impl Limit {
    pub fn ordered(self) -> Option<Limit> {
        match self {
            Limit::Levels { warn, block } => Self::exceeded(warn, block).map_or(Some(self), |_| None),
            Limit::Record(_) => Some(self),
        }
    }

    pub fn judge(&self, measured: Seconds) -> Verdict {
        match self {
            Limit::Levels { warn, block } => Self::leveled(measured, *warn, *block),
            Limit::Record(record) => Self::exceeded(measured, *record).map_or(Verdict::Within, |_| Verdict::Block),
        }
    }

    pub fn lowered(self, measured: Seconds) -> Result<Seconds, Refusal> {
        match self {
            Limit::Levels { .. } => Err(Refusal::Levels),
            Limit::Record(record) => Self::exceeded(measured, record)
                .map_or(Ok(measured), |exceeded| Err(Refusal::Above { record: exceeded })),
        }
    }

    fn leveled(measured: Seconds, warn: Seconds, block: Seconds) -> Verdict {
        Self::exceeded(measured, block)
            .map(|_| Verdict::Block)
            .or_else(|| Self::exceeded(measured, warn).map(|_| Verdict::Warn))
            .unwrap_or(Verdict::Within)
    }

    fn exceeded(measured: Seconds, level: Seconds) -> Option<Seconds> {
        (measured > level).then_some(level)
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
