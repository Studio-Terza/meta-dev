use std::collections::BTreeMap;
use std::path::Path;

use crate::crate_root::CrateRoot;
use lint_layout::LineNumber;
use lint_layout::Manifest;
use lint_layout::Name;
use lint_layout::Untested;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageRoot {
    root: CrateRoot,
    features: BTreeMap<Name, LineNumber>,
    untested: Untested,
}

impl PackageRoot {
    #[must_use]
    pub fn read(directory: &Path) -> PackageRoot {
        let manifest = Manifest::read(directory).ok().flatten();
        PackageRoot {
            root: CrateRoot::of_manifest(directory, manifest.as_ref()),
            features: manifest.as_ref().map(Manifest::features).unwrap_or_default(),
            untested: manifest.as_ref().map(Manifest::untested_targets).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn unplaced(root: CrateRoot) -> PackageRoot {
        PackageRoot { root, features: BTreeMap::new(), untested: Untested::default() }
    }

    #[must_use]
    pub fn root(&self) -> &CrateRoot {
        &self.root
    }

    #[must_use]
    pub fn features(&self) -> &BTreeMap<Name, LineNumber> {
        &self.features
    }

    #[must_use]
    pub fn untested(&self) -> &Untested {
        &self.untested
    }
}
