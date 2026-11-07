use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use file_ports::{FileFailure, Text};
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

    pub fn parsed(path: PathBuf, text: Result<Text, FileFailure>) -> Result<Self, Failure<'static>> {
        match text {
            Ok(text) => {
                serde_json::from_str(text.as_ref()).map_err(|source| Failure::Parse { path: Cow::Owned(path), source })
            }
            Err(failure) => Err(Failure::Io { path: Cow::Owned(path), source: io::Error::from(failure) }),
        }
    }

    pub(crate) fn baseline(&self) -> &[OneLine] {
        &self.baseline
    }

    pub fn mutants(&self) -> &[RecordedMutant] {
        &self.mutants
    }
}
