use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::Bytes;
use port_core::Pass;

use crate::{CargoFailure, CargoFiller, cargo_ops};

#[derive(Clone, Debug, Default)]
pub struct Recorded(BTreeMap<PathBuf, Bytes>);

impl FromIterator<(PathBuf, Bytes)> for Recorded {
    fn from_iter<Answers: IntoIterator<Item = (PathBuf, Bytes)>>(answers: Answers) -> Recorded {
        Recorded(answers.into_iter().collect())
    }
}

impl CargoFiller for Recorded {
    fn metadata(&self, _pass: Pass<'_, cargo_ops::Metadata>, manifest: &Path) -> Result<Bytes, CargoFailure> {
        manifest
            .ancestors()
            .find_map(|recorded| self.0.get(recorded))
            .cloned()
            .ok_or_else(|| CargoFailure::Unrecorded { manifest: manifest.to_path_buf() })
    }
}
