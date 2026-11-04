mod entry;
mod finding;
mod limit;
mod pairing;
mod record;
mod verdict;

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fmt;

use crate::contents::Contents;
use crate::document::Document;
use crate::failure::Failure;
use crate::seconds::Seconds;
use crate::stage::Stage;
pub use finding::Finding;
use limit::Limit;
use newtype::Newtype;
use pairing::Pairing;
pub use record::Record;
use verdict::Verdict;

#[derive(Debug)]
pub struct Gate {
    findings: Vec<Finding>,
}

impl Gate {
    pub fn judge(limits: Contents, measured: Contents, only: &[OsString]) -> Result<Gate, Failure<'_>> {
        let limits: BTreeMap<Stage, Limit> =
            toml::from_str(&limits.into_inner()).map_err(|source| Failure::Parse { file: Document::Limits, source })?;
        let limits = Self::sliced(limits, only)?;
        let measured: BTreeMap<Stage, Seconds> = toml::from_str(&measured.into_inner())
            .map_err(|source| Failure::Parse { file: Document::Measured, source })?;
        Pairing::joined(limits, measured)
            .into_iter()
            .map(|(stage, pairing)| pairing.finding(stage))
            .collect::<Result<Vec<Finding>, Failure<'_>>>()
            .and_then(Self::found)
    }

    fn found(findings: Vec<Finding>) -> Result<Gate, Failure<'static>> {
        findings.first().map(|_| ()).ok_or(Failure::Empty).map(|()| Gate { findings })
    }

    pub fn lowered(self) -> Result<Record, Failure<'static>> {
        self.findings.into_iter().map(Finding::lowered).collect()
    }

    fn sliced(limits: BTreeMap<Stage, Limit>, only: &[OsString]) -> Result<BTreeMap<Stage, Limit>, Failure<'_>> {
        match only.iter().find(|name| limits.keys().find_map(|stage| Self::names(name, stage)).is_none()) {
            Some(name) => Err(Failure::UnknownStage { name }),
            None => Ok(limits.into_iter().filter(|(stage, _)| Self::asked(only, stage).is_some()).collect()),
        }
    }

    fn asked<'stage>(only: &[OsString], stage: &'stage Stage) -> Option<&'stage Stage> {
        only.first().map_or(Some(stage), |_| only.iter().find_map(|name| Self::names(name, stage)))
    }

    fn names<'stage>(name: &OsStr, stage: &'stage Stage) -> Option<&'stage Stage> {
        (name == OsStr::new(stage.as_ref())).then_some(stage)
    }

    #[must_use]
    pub fn blocking(&self, report_only: &[OsString]) -> Option<&Finding> {
        self.findings
            .iter()
            .filter_map(Finding::blocks)
            .find(|finding| report_only.iter().find_map(|name| Self::names(name, finding.stage())).is_none())
    }

    #[must_use]
    pub fn past_warn(self) -> Vec<Finding> {
        self.findings.into_iter().filter(|finding| *AsRef::<Verdict>::as_ref(finding) != Verdict::Within).collect()
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.findings.iter().try_for_each(|finding| writeln!(formatter, "{finding}"))
    }
}
