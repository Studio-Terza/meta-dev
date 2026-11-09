use std::collections::BTreeMap;
use std::ptr;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf};

use crate::StoreFailure;
use crate::sqlite::opened::Opened;

#[derive(Debug, Default)]
pub(super) struct Template {
    image: Mutex<Image>,
}

#[derive(Debug, Default)]
enum Image {
    #[default]
    Unseen,
    Seen,
    Migrated(SqliteOwnedBuf),
}

static TEMPLATES: LazyLock<Mutex<BTreeMap<usize, Arc<Template>>>> = LazyLock::new(Mutex::default);

impl Template {
    pub(super) fn shared(migrator: &'static Migrator) -> Arc<Template> {
        let mut templates = TEMPLATES.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(templates.entry(ptr::from_ref(migrator).addr()).or_default())
    }

    pub(super) fn opened(&self, migrator: &Migrator) -> Result<Opened, StoreFailure> {
        self.image.lock().unwrap_or_else(PoisonError::into_inner).opened(migrator)
    }
}

impl Image {
    fn opened(&mut self, migrator: &Migrator) -> Result<Opened, StoreFailure> {
        match self {
            Image::Unseen => Opened::connected(&Image::memory(), migrator).inspect(|_opened| *self = Image::Seen),
            Image::Seen => Opened::connected(&Image::memory(), migrator).and_then(|mut opened| {
                *self = Image::Migrated(opened.image()?);
                Ok(opened)
            }),
            Image::Migrated(migrated) => SqliteOwnedBuf::try_from(&**migrated)
                .map_err(StoreFailure::Open)
                .and_then(|copied| Opened::copied(&Image::memory(), copied)),
        }
    }

    fn memory() -> SqliteConnectOptions {
        SqliteConnectOptions::new().in_memory(true)
    }
}
