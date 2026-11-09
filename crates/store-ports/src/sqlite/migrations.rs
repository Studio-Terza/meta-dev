use std::any::TypeId;

use newtype::Counter;
use sqlx::migrate::Migrator;

use crate::{Batch, Schema, Statements, StoreFailure, Version, Written};

#[derive(Clone, Copy, Debug)]
pub(super) struct Migrations {
    schema: TypeId,
    migrator: &'static Migrator,
    rewritten: fn(Version, &mut Statements<'_>) -> Result<Written, StoreFailure>,
}

impl Migrations {
    pub(super) fn declared<S: Schema>() -> Migrations {
        Migrations { schema: TypeId::of::<S>(), migrator: S::migrator(), rewritten: Migrations::rewritten_by::<S> }
    }

    pub(super) fn migrator(self) -> &'static Migrator {
        self.migrator
    }

    pub(super) fn schema(self) -> TypeId {
        self.schema
    }

    pub(super) fn rewritten(self, before: Version, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        (self.rewritten)(before, statements)
    }

    fn rewritten_by<S: Schema>(before: Version, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        S::rewrite(before).map_or_else(|| Ok(Written::zero()), |rewrite| rewrite.written(statements))
    }
}
