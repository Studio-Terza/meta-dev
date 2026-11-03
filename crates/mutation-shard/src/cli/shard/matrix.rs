use std::io;
use std::path::Path;

use file_ports::{Bytes, Files};

use serde::Serialize;
use test_log::TestName;

use super::mutant_result::MutantResult;
use crate::cli::shard::failure::Failure;

#[derive(Debug, Serialize)]
pub struct Matrix<'log> {
    baseline: Vec<&'log TestName>,
    mutants: Vec<MutantResult<'log>>,
}

impl<'log> Matrix<'log> {
    pub(crate) fn new(baseline: Vec<&'log TestName>, mutants: Vec<MutantResult<'log>>) -> Matrix<'log> {
        Matrix { baseline, mutants }
    }

    pub fn write(&self, files: &impl Files, destination: &Path) -> Result<(), Failure> {
        let failed = |source| Failure::Parse { path: destination.to_path_buf(), source };
        files
            .write(destination, &Bytes::from(serde_json::to_string(self).map_err(failed)?.as_str()))
            .map_err(|failure| Failure::Io { path: destination.to_path_buf(), source: io::Error::from(failure) })
    }
}
