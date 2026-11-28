use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::time::Duration;

use toml_ports::Toml;

use crate::failure::Failure;
use crate::ports::Disk;

#[derive(Debug, serde::Deserialize)]
struct Limits(BTreeMap<String, Limit>);

#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub(crate) struct Limit {
    warn: f64,
    block: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Within,
    Warned,
    Blocked,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Tenths(Duration);

impl Limit {
    pub(crate) fn read(path: &Path, stage: &str) -> Result<Limit, Failure> {
        let Limits(stages) = Toml::read(&Disk::read(path)?)?;
        stages.get(stage).copied().ok_or_else(|| Failure::Unlimited(stage.to_owned()))
    }

    pub(crate) fn judged(self, spent: Tenths) -> Result<Verdict, Failure> {
        let warn = Duration::try_from_secs_f64(self.warn)?;
        let block = Duration::try_from_secs_f64(self.block)?;
        Ok(if spent.0 > block {
            Verdict::Blocked
        } else if spent.0 > warn {
            Verdict::Warned
        } else {
            Verdict::Within
        })
    }

    pub(crate) fn warn(self) -> f64 {
        self.warn
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
