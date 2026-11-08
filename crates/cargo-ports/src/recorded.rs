use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::Bytes;

use crate::Cargo;

#[derive(Clone, Debug, Default)]
pub struct Recorded(BTreeMap<PathBuf, Bytes>);

impl FromIterator<(PathBuf, Bytes)> for Recorded {
    fn from_iter<Answers: IntoIterator<Item = (PathBuf, Bytes)>>(answers: Answers) -> Recorded {
        Recorded(answers.into_iter().collect())
    }
}

impl Cargo for Recorded {
    fn metadata(&self, manifest: &Path) -> Option<Bytes> {
        self.0.get(manifest).cloned()
    }
}
