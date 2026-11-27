use std::ffi::OsStr;
use std::fmt;
use std::time::{Duration, TryFromFloatSecsError};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub(crate) struct Stage(String);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct Seconds(f64);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct MutantName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Said(String);

impl Stage {
    pub(crate) fn named(stage: &OsStr) -> Stage {
        Stage(stage.to_string_lossy().into_owned())
    }
}

impl Seconds {
    pub(crate) fn duration(self) -> Result<Duration, TryFromFloatSecsError> {
        Duration::try_from_secs_f64(self.0)
    }
}

impl Said {
    pub(crate) fn spoken(text: &str) -> Said {
        Said(text.to_owned())
    }
}

impl MutantName {
    pub(crate) fn line(&self) -> String {
        [self.0.as_str(), "\n"].concat()
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for Said {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
