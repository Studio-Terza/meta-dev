use std::path::{Path, PathBuf};

use lint_layout::Name;

use crate::site::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Harness {
    TestTarget,
    Elsewhere,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileHome {
    krate: Name,
    root: PathBuf,
    module: Vec<Name>,
    harness: Harness,
}

impl FileHome {
    #[must_use]
    pub fn new(krate: Name, root: PathBuf, module: Vec<Name>, harness: Harness) -> FileHome {
        FileHome { krate, root, module, harness }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed<'model, Entity> {
    site: Site<'model>,
    krate: &'model Name,
    root: &'model Path,
    module: &'model [Name],
    harness: Harness,
    entity: Entity,
}

impl<'model, Entity: Copy> Placed<'model, Entity> {
    pub(crate) fn new(site: Site<'model>, home: &'model FileHome, entity: Entity) -> Placed<'model, Entity> {
        Placed { site, krate: &home.krate, root: &home.root, module: &home.module, harness: home.harness, entity }
    }

    #[must_use]
    pub fn site(self) -> Site<'model> {
        self.site
    }

    #[must_use]
    pub fn krate(self) -> &'model Name {
        self.krate
    }

    #[must_use]
    pub fn root(self) -> &'model Path {
        self.root
    }

    #[must_use]
    pub fn module(self) -> &'model [Name] {
        self.module
    }

    #[must_use]
    pub fn harness(self) -> Harness {
        self.harness
    }

    #[must_use]
    pub fn entity(self) -> Entity {
        self.entity
    }
}
