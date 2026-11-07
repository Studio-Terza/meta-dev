use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::crate_name::CrateName;
use crate::crate_root::CrateRoot;
use file_ports::Files;
use lint_layout::Dependencies;
use lint_layout::LineNumber;
use lint_layout::Manifest;
use lint_layout::Name;
use lint_layout::Owned;
use lint_layout::Untested;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageRoot {
    root: CrateRoot,
    name: CrateName,
    features: BTreeMap<Name, LineNumber>,
    untested: Untested,
    owned: Option<Owned>,
    dependencies: Dependencies,
}

impl PackageRoot {
    #[must_use]
    pub fn read_many(files: &impl Files, directories: &[impl AsRef<Path>]) -> Vec<PackageRoot> {
        let manifests: Vec<Option<Manifest>> =
            Manifest::read_many(files, directories).into_iter().map(|read| read.ok().flatten()).collect();
        let named: Vec<PathBuf> = manifests.iter().flatten().flat_map(Manifest::named_paths).collect();
        let found: BTreeMap<PathBuf, PathBuf> = named
            .iter()
            .cloned()
            .zip(files.canonical_many(&named))
            .filter_map(|(path, real)| Some((path, real.ok()?)))
            .collect();
        let canonical = |path: &Path| found.get(path).cloned();
        directories
            .iter()
            .zip(&manifests)
            .map(|(directory, manifest)| {
                let root = CrateRoot::of_manifest(directory.as_ref(), manifest.as_ref());
                PackageRoot {
                    name: CrateName::from(&root),
                    root,
                    features: manifest.as_ref().map(Manifest::features).unwrap_or_default(),
                    untested: manifest
                        .as_ref()
                        .map(|manifest| manifest.untested_targets(canonical))
                        .unwrap_or_default(),
                    owned: manifest.as_ref().and_then(Option::<Owned>::from),
                    dependencies: manifest.as_ref().map(Dependencies::from).unwrap_or_default(),
                }
            })
            .collect()
    }

    #[must_use]
    pub fn unplaced(root: CrateRoot) -> PackageRoot {
        PackageRoot {
            name: CrateName::from(&root),
            root,
            features: BTreeMap::new(),
            untested: Untested::default(),
            owned: None,
            dependencies: Dependencies::default(),
        }
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
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features
    }

    #[must_use]
    pub fn untested(&self) -> &Untested {
        &self.untested
    }

    #[must_use]
    pub fn owned(&self) -> Option<&Owned> {
        self.owned.as_ref()
    }

    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }
}
