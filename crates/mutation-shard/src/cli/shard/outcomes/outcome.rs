mod named;

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use text_lines::LossyText;

use named::Named;
use test_log::{TestLog, TestName};

use crate::cli::shard::mutant_result::MutantResult;

#[derive(Debug, Deserialize)]
pub struct Outcome {
    scenario: Scenario,
    summary: String,
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
    pub(crate) fn baseline(&self, directory: &Path) -> Option<Vec<TestName>> {
        match self.scenario {
            Scenario::Baseline => Some(self.log(directory).names().into_iter().collect()),
            Scenario::Mutant(_) => None,
        }
    }

    pub fn mutant(&self, directory: &Path) -> Option<MutantResult> {
        match &self.scenario {
            Scenario::Mutant(named) => {
                let mut title = String::new();
                write!(title, "{named}").ok();
                Some(MutantResult::new(title, self.summary.clone(), self.log(directory).failed()))
            }
            Scenario::Baseline => None,
        }
    }

    fn log(&self, directory: &Path) -> TestLog {
        self.log_path
            .iter()
            .chain(self.phase_results.iter().filter_map(|phase| phase.log_path.as_ref()))
            .map(|path| directory.join(path))
            .find(|path| path.is_file())
            .and_then(|path| fs::read(path).ok())
            .map(|bytes| TestLog::from(LossyText::from(bytes.as_slice()).as_ref()))
            .unwrap_or_default()
    }
}
