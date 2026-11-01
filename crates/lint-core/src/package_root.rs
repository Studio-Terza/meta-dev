use std::path::{Path, PathBuf};

use crate::crate_root::CrateRoot;
use crate::manifest::Manifest;
use crate::name::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageRoot {
    root: CrateRoot,
    features: Vec<Name>,
}

impl PackageRoot {
    const MANIFEST: &'static str = "Cargo.toml";

    #[must_use]
    pub fn read(directory: &Path) -> PackageRoot {
        let manifest = Manifest::read(directory).ok().flatten();
        PackageRoot {
            root: CrateRoot::of_manifest(directory, manifest.as_ref()),
            features: manifest.as_ref().map(Manifest::feature_names).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn unplaced(root: CrateRoot) -> PackageRoot {
        PackageRoot { root, features: Vec::new() }
    }

    #[must_use]
    pub fn root(&self) -> &CrateRoot {
        &self.root
    }

    #[must_use]
    pub fn features(&self) -> &[Name] {
        &self.features
    }

    #[must_use]
    pub fn manifest_file(&self) -> PathBuf {
        self.root.directory().join(Self::MANIFEST)
    }
}
