use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use file_ports::Files;

use crate::package_root::PackageRoot;
use crate::placement::Placement;

#[derive(Debug, Default)]
pub struct Packages(BTreeMap<PathBuf, PackageRoot>);

impl Packages {
    #[must_use]
    pub fn read(files: &impl Files, packages: &[&Path]) -> Packages {
        let distinct: Vec<&Path> = packages.iter().copied().collect::<BTreeSet<&Path>>().into_iter().collect();
        Packages(
            distinct
                .iter()
                .map(|package| package.to_path_buf())
                .zip(PackageRoot::read_many(files, &distinct))
                .collect(),
        )
    }

    #[must_use]
    pub fn placed(&self, package: &Path, file: &Path) -> Placement {
        Placement::placed(self.0.get(package), package, file)
    }
}
