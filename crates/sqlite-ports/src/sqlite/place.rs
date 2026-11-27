use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::opened::{Opened, Stamped};

use store_ports::{Busy, Claim, StoreFailure, StoreLock};

#[derive(Debug)]
pub(super) enum Place {
    File(PathBuf),
    Memory,
}

impl Place {
    const LOCK: &'static str = "lock";
    const SIDECARS: [&'static str; 2] = ["-wal", "-shm"];

    pub(super) fn opened(
        &self,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
        schema: Stamped,
    ) -> Result<Opened, StoreFailure> {
        match self {
            Place::File(path) => {
                Self::filed(path, prepare)?;
                Opened::filed(path, schema)?.map_or_else(|| Self::rebuilt(path, schema), Ok)
            }
            Place::Memory => Opened::fresh(schema),
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

    fn rebuilt(path: &Path, schema: Stamped) -> Result<Opened, StoreFailure> {
        let files = [path.to_path_buf()].into_iter().chain(Self::SIDECARS.map(|suffix| Self::sidecar(path, suffix)));
        files.into_iter().try_for_each(|file| Self::removed(&file)).map_err(StoreFailure::Stale)?;
        Opened::built(path, schema)
    }

    fn sidecar(path: &Path, suffix: &str) -> PathBuf {
        let mut named = path.as_os_str().to_owned();
        named.push(suffix);
        PathBuf::from(named)
    }

    fn removed(path: &Path) -> io::Result<()> {
        fs::remove_file(path).or_else(|failed| match failed.kind() {
            io::ErrorKind::NotFound => Ok(()),
            _other => Err(failed),
        })
    }
}
