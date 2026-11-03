mod outcome;

use std::io;
use std::path::Path;

use file_ports::Files;

use outcome::Outcome;
pub(crate) use outcome::Title;
use serde::Deserialize;
use test_log::TestName;

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

    pub(crate) fn baseline(&self, files: &impl Files, directory: &Path) -> Vec<TestName> {
        self.outcomes.iter().filter_map(|outcome| outcome.baseline(files, directory)).flatten().collect()
    }

    pub fn mutants(&self, files: &impl Files, directory: &Path) -> Vec<MutantResult> {
        self.outcomes.iter().filter_map(|outcome| outcome.mutant(files, directory)).collect()
    }
}
