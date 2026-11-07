mod locked;
mod reach;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Path, PathBuf};

use file_ports::Files;

use crate::family::Family;
use crate::name::Name;
use crate::owned::Owned;
use locked::Locked;
use reach::Reach;

#[derive(Debug, Default)]
pub struct Locks(BTreeMap<PathBuf, Lock>);

#[derive(Debug, Default)]
struct Lock(Vec<Locked>);

impl Locks {
    const FILE: &'static str = "Cargo.lock";

    #[must_use]
    pub fn read_many(files: &impl Files, directories: &[impl AsRef<Path>]) -> Locks {
        let paths: BTreeSet<PathBuf> = directories
            .iter()
            .flat_map(|directory| directory.as_ref().ancestors().map(|ancestor| ancestor.join(Self::FILE)))
            .collect();
        let paths: Vec<PathBuf> = paths.into_iter().collect();
        Locks(
            paths
                .iter()
                .cloned()
                .zip(files.read_text_many(&paths))
                .filter_map(|(path, read)| Some((path, Lock::from(read.ok()?))))
                .collect(),
        )
    }

    #[must_use]
    pub fn family(&self, directory: &Path, owned: &Owned) -> Family {
        directory
            .ancestors()
            .find_map(|ancestor| self.0.get(&ancestor.join(Self::FILE)))
            .map(|lock| lock.family(owned))
            .unwrap_or_default()
    }
}

impl Lock {
    fn family(&self, owned: &Owned) -> Family {
        owned.as_ref().iter().flat_map(|name| self.kin(name)).collect()
    }

    fn kin(&self, name: &Name) -> Vec<Name> {
        self.0
            .iter()
            .filter_map(|root| root.named(name))
            .flat_map(|root| self.reached(root).into_iter().filter_map(|reached| reached.released_with(root)).cloned())
            .collect()
    }

    fn reached<'lock>(&'lock self, root: &'lock Locked) -> BTreeSet<&'lock Locked> {
        iter::successors(Some(Reach::from(root)), |reach| reach.grown(&self.0))
            .last()
            .map(BTreeSet::from)
            .unwrap_or_default()
    }
}
