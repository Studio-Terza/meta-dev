mod boundary;
mod failed;
mod opened;
mod place;

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use port_core::Pass;

use invariants::Unpoisoned;
use opened::{Opened, Stamped};
use place::Place;
use store_ports::{
    Batch, Busy, Claim, Created, Opens, Query, Record, Schema, StoreFailure, StoreFiller, StoreLock, store_ops,
};

#[derive(Clone, Debug)]
pub struct Sqlite {
    shared: Arc<Shared>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SqliteOpener;

#[must_use]
#[derive(Debug)]
struct Shared {
    place: Place,
    schema: Stamped,
    opened: Mutex<Option<Opened>>,
    claim: Arc<Mutex<Claim>>,
}

impl Sqlite {
    fn placed(schema: &Schema, place: Place) -> Sqlite {
        let shared = Shared { place, schema: Stamped::from(schema), opened: Mutex::default(), claim: Arc::default() };
        Sqlite { shared: Arc::new(shared) }
    }
}

impl Shared {
    fn opened<T, Failure: From<StoreFailure>>(
        &self,
        body: impl FnOnce(&mut Opened) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let mut slot = self.opened.lock().unpoisoned();
        let opened =
            slot.take().map_or_else(|| self.place.opened(|path: &Path| fs::create_dir_all(path), &self.schema), Ok)?;
        body(slot.insert(opened))
    }
}

impl StoreFiller for Sqlite {
    fn read<Q: Query>(&self, _pass: Pass<'_, Q>, query: Q) -> Result<Q::Rows, StoreFailure> {
        self.shared.opened(|opened| opened.read(query))
    }

    fn write<Q: Batch>(&self, _pass: Pass<'_, Q>, batch: Q) -> Result<Q::Done, Q::Failure> {
        self.shared.opened(|opened| opened.transacted(batch))
    }

    fn create<R: Record>(&self, _pass: Pass<'_, Created<R>>, created: Created<R>) -> Result<R, StoreFailure> {
        self.shared.opened(|opened| opened.created(created))
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

impl Opens for SqliteOpener {
    type Store = Sqlite;

    fn file(&self, schema: &Schema, path: PathBuf) -> Sqlite {
        Sqlite::placed(schema, Place::File(path))
    }

    fn in_memory(&self, schema: &Schema) -> Sqlite {
        Sqlite::placed(schema, Place::Memory)
    }
}
