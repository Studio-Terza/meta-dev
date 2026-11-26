mod ledger;

use std::any::TypeId;
use std::collections::BTreeSet;

use rusqlite::Connection;
use store_ports::{Batch, Schema, StoreFailure, Version};

use super::opened::Connected;
use ledger::Ledger;

#[derive(Clone, Copy, Debug)]
pub(super) struct Migrations {
    schema: TypeId,
    declared: &'static store_ports::Migrator,
    rewritten: fn(Version, &mut Connected<'_>) -> Result<(), StoreFailure>,
}

impl Migrations {
    pub(super) fn declared<S: Schema>() -> Migrations {
        Migrations { schema: TypeId::of::<S>(), declared: S::migrator(), rewritten: Migrations::rewritten_by::<S> }
    }

    pub(super) fn schema(self) -> TypeId {
        self.schema
    }

    pub(super) fn applied(self, connection: &Connection) -> Result<BTreeSet<Version>, StoreFailure> {
        Ok(Ledger::read(connection)?.applied(self.declared, connection)?)
    }

    pub(super) fn rewritten(
        self,
        pending: &BTreeSet<Version>,
        statements: &mut Connected<'_>,
    ) -> Result<(), StoreFailure> {
        pending.iter().try_for_each(|version| (self.rewritten)(*version, statements))
    }

    fn rewritten_by<S: Schema>(pending: Version, statements: &mut Connected<'_>) -> Result<(), StoreFailure> {
        S::rewrite(pending).map_or(Ok(()), |rewrite| rewrite.written(statements))
    }
}
