use std::io;
use std::path::Path;

use file_ports::Files;
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

    pub fn read(files: &impl Files, path: &Path) -> Result<ShardMatrix, Failure> {
        let text = files
            .read_text(path)
            .map_err(|failure| Failure::Io { path: path.to_path_buf(), source: io::Error::from(failure) })?;
        serde_json::from_str(text.as_ref()).map_err(|source| Failure::Parse { path: path.to_path_buf(), source })
    }

    pub(crate) fn baseline(&self) -> &[OneLine] {
        &self.baseline
    }

    pub fn mutants(&self) -> &[RecordedMutant] {
        &self.mutants
    }
}
