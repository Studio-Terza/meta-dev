mod finding;
mod limit;
mod seconds;
mod verdict;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::cli::failure::Failure;
use finding::Finding;
use limit::Limit;
use seconds::Seconds;

#[derive(Debug)]
pub struct Gate {
    findings: Vec<Finding>,
}

impl Gate {
    const LIMITS_SOURCE: &'static str = "limits";
    const MEASURED_SOURCE: &'static str = "measured";

    pub fn judge(limits: &str, measured: &str) -> Result<Gate, Failure> {
        let limits: BTreeMap<String, Limit> =
            toml::from_str(limits).map_err(|source| Failure::Parse { file: Self::LIMITS_SOURCE, source })?;
        let measured: BTreeMap<String, Seconds> =
            toml::from_str(measured).map_err(|source| Failure::Parse { file: Self::MEASURED_SOURCE, source })?;
        limits
            .keys()
            .chain(measured.keys())
            .collect::<BTreeSet<&String>>()
            .into_iter()
            .map(|stage| Self::finding(stage, limits.get(stage), measured.get(stage)))
            .collect::<Result<Vec<Finding>, Failure>>()
            .and_then(|findings| if findings.is_empty() { Err(Failure::Empty) } else { Ok(Gate { findings }) })
    }

    #[must_use]
    pub fn blocks(&self) -> bool {
        self.findings.iter().any(Finding::blocks)
    }

    fn finding(stage: &str, limit: Option<&Limit>, measured: Option<&Seconds>) -> Result<Finding, Failure> {
        let limit =
            limit.filter(|limit| limit.is_ordered()).ok_or_else(|| Failure::Unlimited { stage: stage.to_owned() })?;
        let measured = measured.ok_or_else(|| Failure::Unmeasured { stage: stage.to_owned() })?;
        Ok(Finding::new(stage.to_owned(), *measured, *limit))
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.findings.iter().try_for_each(|finding| writeln!(formatter, "{finding}"))
    }
}
