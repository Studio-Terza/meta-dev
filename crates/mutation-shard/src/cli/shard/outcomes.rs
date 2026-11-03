mod outcome;

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Kind};

use outcome::Outcome;
pub(crate) use outcome::Title;
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
        let text = files
            .read_text(&path)
            .map_err(|failure| Failure::Io { path: path.clone(), source: io::Error::from(failure) })?;
        serde_json::from_str(text.as_ref()).map_err(|source| Failure::Parse { path, source })
    }

    pub(crate) fn logs(&self, files: &impl Files, directory: &Path) -> Vec<TestLog> {
        let candidates: Vec<Vec<PathBuf>> = self.outcomes.iter().map(|outcome| outcome.logs(directory)).collect();
        let every: Vec<PathBuf> = candidates.iter().flatten().cloned().collect();
        let kinds: BTreeMap<&PathBuf, Kind> = every.iter().zip(files.kinds(&every)).collect();
        let chosen: Vec<Option<&PathBuf>> =
            candidates.iter().map(|paths| paths.iter().find(|path| kinds.get(path) == Some(&Kind::File))).collect();
        let wanted: Vec<PathBuf> = chosen.iter().flatten().map(|path| (*path).clone()).collect();
        let read: BTreeMap<&PathBuf, Bytes> =
            wanted.iter().zip(files.read_many(&wanted)).filter_map(|(path, bytes)| Some((path, bytes.ok()?))).collect();
        chosen
            .into_iter()
            .map(|path| {
                path.and_then(|path| read.get(path))
                    .map(|bytes| TestLog::from(LossyText::from(bytes.as_ref()).as_ref()))
                    .unwrap_or_default()
            })
            .collect()
    }

    pub(crate) fn baseline(&self, logs: &[TestLog]) -> Vec<TestName> {
        self.outcomes.iter().zip(logs).filter_map(|(outcome, log)| outcome.baseline(log)).flatten().collect()
    }

    pub fn mutants(&self, logs: &[TestLog]) -> Vec<MutantResult> {
        self.outcomes.iter().zip(logs).filter_map(|(outcome, log)| outcome.mutant(log)).collect()
    }
}
