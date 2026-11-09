mod template;

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;

use super::opened::Opened;
use template::Template;

use crate::store_lock::Claim;
use crate::store_lock::StoreLock;
use crate::{Busy, StoreFailure};

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
        migrator: &'static Migrator,
    ) -> Result<Opened, StoreFailure> {
        match self {
            Place::File(path) => Self::filed(path, prepare).and_then(|options| Opened::connected(&options, migrator)),
            Place::Memory => Template::shared(migrator).opened(migrator),
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

    fn filed(path: &Path, prepare: impl FnOnce(&Path) -> io::Result<()>) -> Result<SqliteConnectOptions, StoreFailure> {
        path.parent()
            .map(prepare)
            .transpose()
            .map(|_prepared| SqliteConnectOptions::new().filename(path).create_if_missing(true))
            .map_err(StoreFailure::Directory)
    }
}
