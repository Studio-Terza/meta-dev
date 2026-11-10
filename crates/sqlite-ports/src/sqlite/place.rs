mod template;

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::migrations::Migrations;
use super::opened::Opened;
use template::Template;

use store_ports::{Busy, Claim, StoreFailure, StoreLock};

#[derive(Debug)]
pub(super) enum Place {
    File(PathBuf),
    Memory,
}

impl Place {
    const LOCK: &'static str = "lock";

    pub(super) fn opened(
        &self,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
        migrations: Migrations,
    ) -> Result<Opened, StoreFailure> {
        match self {
            Place::File(path) => Self::filed(path, prepare).and_then(|()| Opened::filed(path, migrations)),
            Place::Memory => Template::shared(migrations).opened(migrations),
        }
    }

    pub(super) fn returned(&self, opened: Option<Opened>, migrations: Migrations) {
        match self {
            Place::File(_) => drop(opened),
            Place::Memory => opened.into_iter().for_each(|idle| Template::shared(migrations).returned(idle)),
        }
    }

    pub(super) fn locked(
        &self,
        claim: &Arc<Mutex<Claim>>,
        open: impl FnOnce(&Path) -> io::Result<File>,
    ) -> Result<StoreLock, Busy> {
        match self {
            Place::File(path) => StoreLock::filed(path.with_added_extension(Self::LOCK), open),
            Place::Memory => StoreLock::claimed(claim),
        }
    }

    fn filed(path: &Path, prepare: impl FnOnce(&Path) -> io::Result<()>) -> Result<(), StoreFailure> {
        path.parent().map(prepare).transpose().map(|_prepared| ()).map_err(StoreFailure::Directory)
    }
}
