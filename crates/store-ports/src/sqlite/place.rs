use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sqlx::sqlite::SqliteConnectOptions;

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

    pub(super) fn connecting(
        &self,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
    ) -> Result<SqliteConnectOptions, StoreFailure> {
        match self {
            Place::File(path) => Self::filed(path, prepare),
            Place::Memory => Ok(SqliteConnectOptions::new().in_memory(true)),
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
