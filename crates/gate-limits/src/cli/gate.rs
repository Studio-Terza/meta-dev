mod finding;
mod limit;
mod record;
mod seconds;
mod verdict;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt;

use crate::cli::contents::Contents;
use crate::cli::document::Document;
use crate::cli::failure::Failure;
use crate::cli::stage::Stage;
pub use finding::{Blocking, Finding};
use limit::Limit;
use newtype::Newtype;
pub use record::Record;
pub(crate) use seconds::Seconds;

#[derive(Debug)]
pub struct Gate {
    findings: Vec<Finding>,
}

impl Gate {
    pub fn judge(limits: Contents, measured: Contents, only: &[OsString]) -> Result<Gate, Failure> {
        let limits: BTreeMap<Stage, Limit> =
            toml::from_str(&limits.into_inner()).map_err(|source| Failure::Parse { file: Document::Limits, source })?;
        let limits = Self::sliced(limits, only)?;
        let measured: BTreeMap<Stage, Seconds> = toml::from_str(&measured.into_inner())
            .map_err(|source| Failure::Parse { file: Document::Measured, source })?;
        limits
            .keys()
            .chain(measured.keys())
            .collect::<BTreeSet<&Stage>>()
            .into_iter()
            .map(|stage| Finding::new(stage, limits.get(stage), measured.get(stage)))
            .collect::<Result<Vec<Finding>, Failure>>()
            .and_then(|findings| if findings.is_empty() { Err(Failure::Empty) } else { Ok(Gate { findings }) })
    }

    pub fn lowered(self) -> Result<Record, Failure> {
        self.findings.into_iter().map(Finding::lowered).collect()
    }

    fn sliced(limits: BTreeMap<Stage, Limit>, only: &[OsString]) -> Result<BTreeMap<Stage, Limit>, Failure> {
        match only.iter().find(|name| limits.keys().find_map(|stage| Self::names(name, stage)).is_none()) {
            Some(name) => Err(Failure::UnknownStage { name: name.clone() }),
            None => Ok(limits.into_iter().filter(|(stage, _)| Self::asked(only, stage).is_some()).collect()),
        }
    }

    fn asked<'stage>(only: &[OsString], stage: &'stage Stage) -> Option<&'stage Stage> {
        match only {
            [] => Some(stage),
            named => named.iter().find_map(|name| Self::names(name, stage)),
        }
    }

    fn names<'stage>(name: &OsStr, stage: &'stage Stage) -> Option<&'stage Stage> {
        (name == OsStr::new(stage.as_ref())).then_some(stage)
    }

    #[must_use]
    pub fn blocking(&self, report_only: &[OsString]) -> Option<Blocking<'_>> {
        self.findings
            .iter()
            .filter_map(Finding::blocks)
            .find(|blocking| report_only.iter().find_map(|name| Self::names(name, blocking.stage())).is_none())
    }

    #[must_use]
    pub fn past_warn(self) -> Vec<Finding> {
        self.findings.into_iter().filter_map(Finding::past_warn).collect()
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.findings.iter().try_for_each(|finding| writeln!(formatter, "{finding}"))
    }
}
