use std::fs;
use std::path::Path;

use serde::Serialize;
use test_log::TestName;

use super::mutant_result::MutantResult;
use crate::cli::shard::failure::Failure;

#[derive(Debug, Serialize)]
pub struct Matrix {
    baseline: Vec<TestName>,
    mutants: Vec<MutantResult>,
}

impl Matrix {
    pub(crate) fn new(baseline: Vec<TestName>, mutants: Vec<MutantResult>) -> Matrix {
        Matrix { baseline, mutants }
    }

    pub fn write(&self, destination: &Path) -> Result<(), Failure> {
        let failed = |source| Failure::Parse { path: destination.to_path_buf(), source };
        fs::write(destination, serde_json::to_string(self).map_err(failed)?)
            .map_err(|source| Failure::Io { path: destination.to_path_buf(), source })
    }
}
