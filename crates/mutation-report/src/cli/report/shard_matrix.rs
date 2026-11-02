use std::fs;
use std::path::Path;

use serde::Deserialize;
use text_lines::OneLine;

use super::recorded_mutant::RecordedMutant;
use crate::cli::report::failure::Failure;

#[derive(Debug, Deserialize)]
pub struct ShardMatrix {
    baseline: Vec<OneLine>,
    mutants: Vec<RecordedMutant>,
}

impl ShardMatrix {
    pub const FILE: &'static str = "matrix.json";

    pub fn read(path: &Path) -> Result<ShardMatrix, Failure> {
        let text = fs::read_to_string(path).map_err(|source| Failure::Io { path: path.to_path_buf(), source })?;
        serde_json::from_str(&text).map_err(|source| Failure::Parse { path: path.to_path_buf(), source })
    }

    pub(crate) fn baseline(&self) -> &[OneLine] {
        &self.baseline
    }

    pub fn mutants(&self) -> &[RecordedMutant] {
        &self.mutants
    }
}
