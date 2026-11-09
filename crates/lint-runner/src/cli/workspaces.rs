use std::path::PathBuf;

use cargo_ports::Cargo;
use lint_reader::{CrateSources, Unlocated};
use workspace_metadata::Metadata;

#[derive(Debug, Default)]
pub(crate) struct Workspaces(Vec<Metadata>);

impl Workspaces {
    const MANIFEST: &'static str = "Cargo.toml";

    pub(crate) fn described(cargo: &impl Cargo, roots: &[PathBuf]) -> Result<Workspaces, Unlocated> {
        let manifests: Vec<PathBuf> = roots.iter().map(|root| root.join(Self::MANIFEST)).collect();
        CrateSources::described(cargo, &manifests).map(Workspaces)
    }
}

impl AsRef<[Metadata]> for Workspaces {
    fn as_ref(&self) -> &[Metadata] {
        &self.0
    }
}
