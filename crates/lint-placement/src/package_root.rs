use std::path::Path;

use crate::crate_name::CrateName;
use crate::crate_root::CrateRoot;
use crate::ignored::Ignored;
use file_ports::Files;
use invariants::Ignoring;
use lint_layout::KeyLines;
use lint_layout::Manifest;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageRoot {
    root: CrateRoot,
    name: CrateName,
    lines: KeyLines,
}

impl PackageRoot {
    #[must_use]
    pub fn read_many(files: &impl Files, directories: &[impl AsRef<Path>]) -> Vec<PackageRoot> {
        directories
            .iter()
            .zip(Manifest::read_many(files, directories))
            .map(|(directory, read)| {
                let manifest = read.ignoring(Ignored::NotAManifest).flatten();
                let root = CrateRoot::of_manifest(directory.as_ref(), manifest.as_ref());
                PackageRoot {
                    name: CrateName::from(&root),
                    root,
                    lines: manifest.as_ref().map(AsRef::as_ref).cloned().unwrap_or_default(),
                }
            })
            .collect()
    }

    #[must_use]
    pub fn unplaced(root: CrateRoot) -> PackageRoot {
        PackageRoot { name: CrateName::from(&root), root, lines: KeyLines::default() }
    }

    #[must_use]
    pub fn root(&self) -> &CrateRoot {
        &self.root
    }

    #[must_use]
    pub fn crate_name(&self) -> &CrateName {
        &self.name
    }

    #[must_use]
    pub fn key_lines(&self) -> &KeyLines {
        &self.lines
    }
}
