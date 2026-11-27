mod ledger;

use std::any::TypeId;
use std::collections::BTreeSet;

use crate::store::{Migrator, Schema, StoreFailure, Version};
use ledger::Ledger;
use rusqlite::Connection;

#[derive(Clone, Copy, Debug)]
pub(super) struct Migrations {
    schema: TypeId,
    declared: &'static Migrator,
}

impl Migrations {
    pub(super) fn declared<S: Schema>() -> Migrations {
        Migrations { schema: TypeId::of::<S>(), declared: S::migrator() }
    }

    pub(super) fn schema(self) -> TypeId {
        self.schema
    }

    pub(super) fn applied(self, connection: &Connection) -> Result<BTreeSet<Version>, StoreFailure> {
        Ok(Ledger::read(connection)?.applied(self.declared, connection)?)
    }
}
