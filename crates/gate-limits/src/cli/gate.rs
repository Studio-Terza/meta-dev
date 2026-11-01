mod finding;
mod limit;
mod seconds;
mod verdict;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt;

use crate::cli::contents::Contents;
use crate::cli::document::Document;
use crate::cli::failure::Failure;
use crate::cli::stage::Stage;
pub use finding::Finding;
use limit::Limit;
use newtype::Newtype;
use seconds::Seconds;

#[derive(Debug)]
pub struct Gate {
    findings: Vec<Finding>,
}

impl Gate {
    pub fn judge(limits: Contents<'_>, measured: Contents<'_>, only: &[OsString]) -> Result<Gate, Failure> {
        let limits: BTreeMap<Stage, Limit> =
            toml::from_str(limits.into_inner()).map_err(|source| Failure::Parse { file: Document::Limits, source })?;
        let limits = Self::sliced(limits, only)?;
        let measured: BTreeMap<Stage, Seconds> = toml::from_str(measured.into_inner())
            .map_err(|source| Failure::Parse { file: Document::Measured, source })?;
        limits
            .keys()
            .chain(measured.keys())
            .collect::<BTreeSet<&Stage>>()
            .into_iter()
            .map(|stage| Self::finding(stage, limits.get(stage), measured.get(stage)))
            .collect::<Result<Vec<Finding>, Failure>>()
            .and_then(|findings| if findings.is_empty() { Err(Failure::Empty) } else { Ok(Gate { findings }) })
    }

    fn sliced(limits: BTreeMap<Stage, Limit>, only: &[OsString]) -> Result<BTreeMap<Stage, Limit>, Failure> {
        match only.iter().find(|name| !limits.keys().any(|stage| Self::names(name, stage))) {
            Some(name) => Err(Failure::UnknownStage { name: name.clone() }),
            None => Ok(limits.into_iter().filter(|(stage, _)| Self::asked(only, stage)).collect()),
        }
    }

    fn asked(only: &[OsString], stage: &Stage) -> bool {
        only.is_empty() || only.iter().any(|name| Self::names(name, stage))
    }

    fn names(name: &OsStr, stage: &Stage) -> bool {
        name == OsStr::new(stage.as_ref())
    }

    #[must_use]
    pub fn blocks(&self) -> bool {
        self.findings.iter().any(Finding::blocks)
    }

    #[must_use]
    pub fn past_warn(self) -> Vec<Finding> {
        self.findings.into_iter().filter_map(Finding::past_warn).collect()
    }

    fn finding(stage: &Stage, limit: Option<&Limit>, measured: Option<&Seconds>) -> Result<Finding, Failure> {
        let limit =
            limit.filter(|limit| limit.is_ordered()).ok_or_else(|| Failure::Unlimited { stage: stage.clone() })?;
        let measured = measured.ok_or_else(|| Failure::Unmeasured { stage: stage.clone() })?;
        Ok(Finding::new(stage.clone(), *measured, *limit))
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.findings.iter().try_for_each(|finding| writeln!(formatter, "{finding}"))
    }
}
