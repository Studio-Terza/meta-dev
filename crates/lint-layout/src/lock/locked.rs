mod boundary;
mod release;

use crate::name::Name;
use release::{Release, Version};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Locked {
    name: Name,
    release: Release,
    requires: Vec<Required>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Required {
    name: Name,
    version: Option<Version>,
}

impl Locked {
    pub(super) fn named(&self, name: &Name) -> Option<&Locked> {
        Some(self).filter(|locked| locked.name == *name)
    }

    pub(super) fn released_with(&self, root: &Locked) -> Option<&Name> {
        self.release.shared(&root.release).map(|_| &self.name)
    }

    pub(super) fn requirements<'lock>(&self, lock: &'lock [Locked]) -> Vec<&'lock Locked> {
        self.requires.iter().flat_map(|required| lock.iter().filter_map(|locked| required.picks(locked))).collect()
    }
}

impl Required {
    fn picks<'lock>(&self, locked: &'lock Locked) -> Option<&'lock Locked> {
        locked.named(&self.name).filter(|locked| locked.release.versioned(self.version.as_ref()).is_some())
    }
}
