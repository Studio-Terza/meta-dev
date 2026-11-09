use std::any::TypeId;
use std::borrow::Cow;
use std::collections::BTreeSet;

use newtype::Counter;
use sqlx::SqlStr;
use sqlx::migrate::{Migration, MigrationType, Migrator};
use store_ports::{Batch, Schema, Statements, StoreFailure, Version, Written};

#[derive(Clone, Copy, Debug)]
pub(super) struct Migrations {
    schema: TypeId,
    declared: &'static store_ports::Migrator,
    rewritten: fn(Version, &mut Statements<'_>) -> Result<Written, StoreFailure>,
}

impl Migrations {
    pub(super) fn declared<S: Schema>() -> Migrations {
        Migrations { schema: TypeId::of::<S>(), declared: S::migrator(), rewritten: Migrations::rewritten_by::<S> }
    }

    pub(super) fn migrator(self) -> Migrator {
        let ignored = bool::from(self.declared.missing());
        let migrator = Migrator::with_migrations(self.declared.iter().map(Self::migration).collect());
        Migrator { ignore_missing: ignored, ..migrator }
    }

    pub(super) fn schema(self) -> TypeId {
        self.schema
    }

    pub(super) fn rewritten(
        self,
        pending: &BTreeSet<Version>,
        statements: &mut Statements<'_>,
    ) -> Result<Written, StoreFailure> {
        let written = pending
            .iter()
            .map(|version| (self.rewritten)(*version, statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }

    fn migration(declared: &store_ports::Migration) -> Migration {
        declared.rebuilt(|version, description, sql, transacted| {
            Migration::new(
                version,
                Cow::Borrowed(description),
                MigrationType::Simple,
                SqlStr::from_static(sql),
                bool::from(transacted),
            )
        })
    }

    fn rewritten_by<S: Schema>(pending: Version, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        S::rewrite(pending).map_or_else(|| Ok(Written::zero()), |rewrite| rewrite.written(statements))
    }
}
