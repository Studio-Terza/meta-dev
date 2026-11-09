use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use crate::sqlite::migrations::Migrations;
use crate::sqlite::opened::{Opened, Snapshot};
use store_ports::StoreFailure;

#[derive(Debug, Default)]
pub(super) struct Template {
    image: Mutex<Image>,
}

#[derive(Debug, Default)]
enum Image {
    #[default]
    Unseen,
    Seen,
    Migrated(Snapshot),
}

static TEMPLATES: LazyLock<Mutex<BTreeMap<TypeId, Arc<Template>>>> = LazyLock::new(Mutex::default);

impl Template {
    pub(super) fn shared(migrations: Migrations) -> Arc<Template> {
        let mut templates = TEMPLATES.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(templates.entry(migrations.schema()).or_default())
    }

    pub(super) fn opened(&self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        self.image.lock().unwrap_or_else(PoisonError::into_inner).opened(migrations)
    }
}

impl Image {
    fn opened(&mut self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        match self {
            Image::Unseen => Opened::fresh(migrations).inspect(|_opened| *self = Image::Seen),
            Image::Seen => Opened::fresh(migrations).and_then(|opened| {
                *self = Image::Migrated(opened.image()?);
                Ok(opened)
            }),
            Image::Migrated(migrated) => Opened::copied(migrated),
        }
    }
}
