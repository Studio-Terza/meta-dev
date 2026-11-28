use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use pipeline::{Limit, Verdict};
use serde::Deserialize;

use crate::failure::Failure;
use crate::ports::Disk;

#[derive(Debug, Deserialize)]
struct Limits(BTreeMap<String, Stage>);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    block: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct Tenths(Duration);

impl Stage {
    pub(crate) fn read(path: &Path, stage: &str) -> Result<Stage, Failure> {
        let Limits(stages) = toml::from_str(&Disk::read(path)?)?;
        stages.get(stage).copied().ok_or_else(|| Failure::Unlimited(stage.to_owned()))
    }

    pub(crate) fn judged(self, spent: Tenths) -> Result<Verdict, Failure> {
        Ok(Limit::at(Tenths(Duration::try_from_secs_f64(self.block)?)).judged(&spent))
    }

    pub(crate) fn block(self) -> f64 {
        self.block
    }
}

impl From<Duration> for Tenths {
    fn from(spent: Duration) -> Tenths {
        let tenths = u64::from(spent.subsec_millis() / Tenths::MILLIS);
        Tenths(Duration::from_secs(spent.as_secs()) + Duration::from_millis(tenths * u64::from(Tenths::MILLIS)))
    }
}

impl Tenths {
    const MILLIS: u32 = 100;
}

impl fmt::Display for Tenths {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.1} s", self.0.as_secs_f64())
    }
}
