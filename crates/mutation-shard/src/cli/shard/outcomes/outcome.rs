mod named;
mod title;

use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use serde::Deserialize;
use text_lines::LossyText;

use named::Named;
use test_log::{TestLog, TestName};
pub use title::Title;

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
    pub(crate) fn baseline(&self, files: &impl Files, directory: &Path) -> Option<Vec<TestName>> {
        match self.scenario {
            Scenario::Baseline => Some(self.log(files, directory).names().into_iter().collect()),
            Scenario::Mutant(_) => None,
        }
    }

    pub fn mutant(&self, files: &impl Files, directory: &Path) -> Option<MutantResult> {
        match &self.scenario {
            Scenario::Mutant(named) => {
                Some(MutantResult::new(Title::from(named), self.summary, self.log(files, directory).failed()))
            }
            Scenario::Baseline => None,
        }
    }

    fn log(&self, files: &impl Files, directory: &Path) -> TestLog {
        self.log_path
            .iter()
            .chain(self.phase_results.iter().filter_map(|phase| phase.log_path.as_ref()))
            .map(|path| directory.join(path))
            .find(|path| files.kind(path) == Kind::File)
            .and_then(|path| files.read(&path).ok())
            .map(|bytes| TestLog::from(LossyText::from(bytes.as_ref()).as_ref()))
            .unwrap_or_default()
    }
}
