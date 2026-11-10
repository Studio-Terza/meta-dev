use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock, PoisonError};

use crate::sqlite::migrations::Migrations;
use crate::sqlite::opened::{Emptying, Opened, Snapshot};
use store_ports::StoreFailure;

#[derive(Debug, Default)]
pub(super) struct Template {
    image: Mutex<Image>,
    idle: Mutex<Vec<Opened>>,
    emptying: OnceLock<Option<Emptying>>,
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

    const IDLE: usize = 64;

    pub(super) fn opened(&self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        let reused = self.idle.lock().unwrap_or_else(PoisonError::into_inner).pop();
        reused.map_or_else(|| self.image.lock().unwrap_or_else(PoisonError::into_inner).opened(migrations), Ok)
    }

    pub(super) fn returned(&self, opened: Opened) {
        let emptying = self.emptying.get_or_init(|| opened.emptying());
        let emptied = emptying.as_ref().and_then(|tables| opened.emptied(tables));
        let mut idle = self.idle.lock().unwrap_or_else(PoisonError::into_inner);
        let free = Self::IDLE.saturating_sub(idle.len());
        idle.extend(emptied.into_iter().take(free));
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
