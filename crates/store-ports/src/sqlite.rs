mod migrations;
mod opened;
mod place;

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use port_core::Pass;

use crate::store::{StoreFiller, store_ops};
use crate::store_lock::Claim;
use crate::store_lock::StoreLock;
use crate::{Batch, Busy, Query, Schema, StoreFailure, Written};
use migrations::Migrations;
use opened::Opened;
use place::Place;

#[derive(Clone, Debug)]
pub struct Sqlite {
    shared: Arc<Shared>,
}

#[derive(Debug)]
#[must_use]
struct Shared {
    place: Place,
    migrations: Migrations,
    opened: Mutex<Option<Opened>>,
    claim: Arc<Mutex<Claim>>,
}

impl Sqlite {
    #[must_use]
    pub fn file<S: Schema>(path: PathBuf) -> Sqlite {
        Sqlite::placed(Place::File(path), Migrations::declared::<S>())
    }

    #[must_use]
    pub fn in_memory<S: Schema>() -> Sqlite {
        Sqlite::placed(Place::Memory, Migrations::declared::<S>())
    }

    fn placed(place: Place, migrations: Migrations) -> Sqlite {
        let shared = Shared { place, migrations, opened: Mutex::default(), claim: Arc::default() };
        Sqlite { shared: Arc::new(shared) }
    }
}

impl Shared {
    fn opened<T, Failure: From<StoreFailure>>(
        &self,
        prepare: impl FnOnce(&Path) -> io::Result<()>,
        body: impl FnOnce(&mut Opened) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let mut slot = self.opened.lock().unwrap_or_else(PoisonError::into_inner);
        let opened = slot.take().map_or_else(|| self.place.opened(prepare, self.migrations), Ok)?;
        body(slot.insert(opened))
    }
}

impl StoreFiller for Sqlite {
    fn read<Q: Query>(&self, _pass: Pass<'_, Q>, query: Q) -> Result<Q::Rows, StoreFailure> {
        self.shared.opened(|path: &Path| fs::create_dir_all(path), |opened| opened.read(query))
    }

    fn write<Q: Batch>(&self, _pass: Pass<'_, Q>, batch: Q) -> Result<Written, Q::Failure> {
        self.shared.opened(|path: &Path| fs::create_dir_all(path), |opened| opened.transacted(batch))
    }

    fn lock(&self, _pass: Pass<'_, store_ops::Lock>) -> Result<StoreLock, Busy> {
        self.shared.place.locked(&self.shared.claim, |path| {
            path.parent()
                .map(fs::create_dir_all)
                .transpose()
                .and_then(|_created| OpenOptions::new().create(true).truncate(false).write(true).open(path))
        })
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        let opened = self.opened.get_mut().ok().and_then(Option::take);
        self.place.kept(self.migrations, opened);
    }
}
