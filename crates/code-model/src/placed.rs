use std::path::{Path, PathBuf};

use lint_layout::Name;

use crate::site::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileHome {
    krate: Name,
    root: PathBuf,
    module: Vec<Name>,
}

impl FileHome {
    #[must_use]
    pub fn new(krate: Name, root: PathBuf, module: Vec<Name>) -> FileHome {
        FileHome { krate, root, module }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed<'model, Entity> {
    site: Site<'model>,
    krate: &'model Name,
    root: &'model Path,
    module: &'model [Name],
    entity: Entity,
}

impl<'model, Entity: Copy> Placed<'model, Entity> {
    pub(crate) fn new(site: Site<'model>, home: &'model FileHome, entity: Entity) -> Placed<'model, Entity> {
        Placed { site, krate: &home.krate, root: &home.root, module: &home.module, entity }
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
    pub fn entity(self) -> Entity {
        self.entity
    }
}
