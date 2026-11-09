use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf};

use crate::StoreFailure;
use crate::sqlite::migrations::Migrations;
use crate::sqlite::opened::Opened;

#[derive(Debug, Default)]
pub(super) struct Template {
    image: Mutex<Image>,
    spare: Mutex<Vec<Opened>>,
}

#[derive(Debug, Default)]
enum Image {
    #[default]
    Unseen,
    Seen,
    Migrated(SqliteOwnedBuf),
}

static TEMPLATES: LazyLock<Mutex<BTreeMap<TypeId, Arc<Template>>>> = LazyLock::new(Mutex::default);

impl Template {
    pub(super) fn shared(migrations: Migrations) -> Arc<Template> {
        let mut templates = TEMPLATES.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(templates.entry(migrations.schema()).or_default())
    }

    pub(super) fn opened(&self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        let spare = self.spare.lock().unwrap_or_else(PoisonError::into_inner).pop();
        spare.map_or_else(
            || self.image.lock().unwrap_or_else(PoisonError::into_inner).opened(migrations).and_then(Opened::rewound),
            Ok,
        )
    }

    pub(super) fn returned(&self, opened: Opened) {
        let rewound = opened.rewound();
        self.spare.lock().unwrap_or_else(PoisonError::into_inner).extend(rewound);
    }
}

impl Image {
    fn opened(&mut self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        match self {
            Image::Unseen => Opened::connected(&Image::memory(), migrations).inspect(|_opened| *self = Image::Seen),
            Image::Seen => Opened::connected(&Image::memory(), migrations).and_then(|mut opened| {
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
