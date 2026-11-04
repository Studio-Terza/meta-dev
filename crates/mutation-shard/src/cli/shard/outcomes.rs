mod outcome;

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, FileFailure, Files, Kind, Text};

use outcome::Outcome;
use serde::Deserialize;
use test_log::{TestLog, TestName};
use text_lines::LossyText;

use super::mutant_result::MutantResult;
use crate::cli::shard::failure::Failure;

#[derive(Debug, Deserialize)]
pub struct Outcomes {
    outcomes: Vec<Outcome>,
}

impl Outcomes {
    const FILE: &'static str = "outcomes.json";

    pub fn read(files: &impl Files, directory: &Path) -> Result<Outcomes, Failure> {
        let path = directory.join(Self::FILE);
        Self::parsed(files.read_text(&path), path)
    }

    fn parsed(read: Result<Text, FileFailure>, path: PathBuf) -> Result<Outcomes, Failure> {
        match read {
            Ok(text) => serde_json::from_str(text.as_ref()).map_err(|source| Failure::Parse { path, source }),
            Err(failure) => Err(Failure::Io { path, source: io::Error::from(failure) }),
        }
    }

    pub(crate) fn logs(&self, files: &impl Files, directory: &Path) -> Vec<TestLog> {
        let candidates: Vec<Vec<PathBuf>> = self.outcomes.iter().map(|outcome| outcome.logs(directory)).collect();
        let every: Vec<&PathBuf> = candidates.iter().flatten().collect();
        let kinds: BTreeMap<&PathBuf, Kind> = every.iter().copied().zip(files.kinds(&every)).collect();
        let chosen: Vec<Option<&PathBuf>> =
            candidates.iter().map(|paths| paths.iter().find(|path| kinds.get(path) == Some(&Kind::File))).collect();
        let wanted: Vec<&PathBuf> = chosen.iter().flatten().copied().collect();
        let read: BTreeMap<&PathBuf, Bytes> = wanted
            .iter()
            .copied()
            .zip(files.read_many(&wanted))
            .filter_map(|(path, bytes)| Some((path, bytes.ok()?)))
            .collect();
        chosen
            .into_iter()
            .map(|path| {
                path.and_then(|path| read.get(path))
                    .map(|bytes| TestLog::from(LossyText::from(bytes.as_ref()).as_ref()))
                    .unwrap_or_default()
            })
            .collect()
    }

    pub(crate) fn baseline<'log>(&self, logs: &'log [TestLog]) -> Vec<&'log TestName> {
        self.outcomes.iter().zip(logs).filter_map(|(outcome, log)| outcome.baseline(log)).flatten().collect()
    }

    pub fn mutants<'log>(&self, logs: &'log [TestLog]) -> Vec<MutantResult<'log>> {
        self.outcomes.iter().zip(logs).filter_map(|(outcome, log)| outcome.mutant(log)).collect()
    }
}
