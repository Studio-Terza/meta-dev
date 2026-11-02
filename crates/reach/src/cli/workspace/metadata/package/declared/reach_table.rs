use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReachTable {
    reads: Vec<PathBuf>,
}

impl AsRef<[PathBuf]> for ReachTable {
    fn as_ref(&self) -> &[PathBuf] {
        &self.reads
    }
}
