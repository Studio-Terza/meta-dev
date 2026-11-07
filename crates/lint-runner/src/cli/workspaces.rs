use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use cargo_ports::Cargo;
use file_ports::Text;
use lint_layout::Failure;
use workspace_metadata::Metadata;

#[derive(Debug, Default)]
pub(crate) struct Workspaces(Vec<Metadata>);

impl Workspaces {
    pub(crate) fn described(cargo: &impl Cargo, roots: &[PathBuf]) -> Result<Workspaces, Failure> {
        roots
            .iter()
            .zip(cargo.metadata(roots))
            .map(|(root, described)| Self::parsed(root, described))
            .collect::<Result<Vec<Metadata>, Failure>>()
            .map(Workspaces)
    }

    fn parsed(root: &Path, described: io::Result<Text>) -> Result<Metadata, Failure> {
        described.map_err(|source| Failure::Cargo { path: root.to_path_buf(), source }).and_then(|text| {
            Metadata::from_str(text.as_ref()).map_err(|source| Failure::Metadata { path: root.to_path_buf(), source })
        })
    }
}

impl AsRef<[Metadata]> for Workspaces {
    fn as_ref(&self) -> &[Metadata] {
        &self.0
    }
}
