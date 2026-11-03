mod named;

use std::path::{Path, PathBuf};

use serde::Deserialize;

use named::Named;
use test_log::{TestLog, TestName};

use crate::cli::shard::mutant_result::MutantResult;
use crate::cli::shard::verdict::Verdict;

#[derive(Debug, Deserialize)]
pub struct Outcome {
    scenario: Scenario,
    summary: Verdict,
    log_path: Option<PathBuf>,
    #[serde(default)]
    phase_results: Vec<Phase>,
}

#[derive(Debug, Deserialize)]
struct Phase {
    log_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
enum Scenario {
    Baseline,
    Mutant(Named),
}

impl Outcome {
    pub(crate) fn baseline<'log>(&self, log: &'log TestLog) -> Option<Vec<&'log TestName>> {
        match self.scenario {
            Scenario::Baseline => Some(log.names().collect()),
            Scenario::Mutant(_) => None,
        }
    }

    pub fn mutant<'log>(&self, log: &'log TestLog) -> Option<MutantResult<'log>> {
        match &self.scenario {
            Scenario::Mutant(named) => Some(MutantResult::new(named.title(), self.summary, log.failed())),
            Scenario::Baseline => None,
        }
    }

    pub(crate) fn logs(&self, directory: &Path) -> Vec<PathBuf> {
        self.log_path
            .iter()
            .chain(self.phase_results.iter().filter_map(|phase| phase.log_path.as_ref()))
            .map(|path| directory.join(path))
            .collect()
    }
}
