use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::seconds::Seconds;
use crate::cli::command::failure::Failure;

#[derive(Clone, Debug)]
pub struct Measured(PathBuf);

impl Measured {
    pub fn file(path: &Path) -> Measured {
        Measured(path.to_path_buf())
    }

    pub fn append(&self, stage: &str, seconds: Seconds) -> Result<(), Failure> {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.0)
            .and_then(|mut file| writeln!(file, "{stage} = {seconds}"))
            .map_err(|source| Failure::Io { path: self.0.clone(), source })
    }
}
