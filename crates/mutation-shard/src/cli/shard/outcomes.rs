mod outcome;

use std::fs;
use std::path::Path;

use outcome::Outcome;
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

    pub fn read(directory: &Path) -> Result<Outcomes, Failure> {
        let path = directory.join(Self::FILE);
        let text = fs::read_to_string(&path).map_err(|source| Failure::Io { path: path.clone(), source })?;
        serde_json::from_str(&text).map_err(|source| Failure::Parse { path, source })
    }

    pub(crate) fn baseline(&self, directory: &Path) -> Vec<TestName> {
        self.outcomes.iter().filter_map(|outcome| outcome.baseline(directory)).flatten().collect()
    }

    pub fn mutants(&self, directory: &Path) -> Vec<MutantResult> {
        self.outcomes.iter().filter_map(|outcome| outcome.mutant(directory)).collect()
    }
}
