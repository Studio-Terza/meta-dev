use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use pipeline::{Limit, Verdict};
use toml_ports::Toml;

use crate::failure::Failure;
use crate::ports::Disk;

#[derive(Debug, serde::Deserialize)]
struct Limits(BTreeMap<String, Stage>);

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    block: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct Took(Duration);

impl Stage {
    pub(crate) fn read(path: &Path, stage: &str) -> Result<Stage, Failure> {
        let Limits(stages) = Toml::read(&Disk::read(path)?)?;
        stages.get(stage).copied().ok_or_else(|| Failure::Unlimited(stage.to_owned()))
    }

    pub(crate) fn judged(self, spent: Took) -> Result<Verdict, Failure> {
        Ok(Limit::at(Took(Duration::try_from_secs_f64(self.block)?)).judged(&spent))
    }

    pub(crate) fn block(self) -> f64 {
        self.block
    }
}

impl From<Duration> for Took {
    fn from(spent: Duration) -> Took {
        Took(spent)
    }
}

impl fmt::Display for Took {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.1} s", self.0.as_secs_f64())
    }
}
