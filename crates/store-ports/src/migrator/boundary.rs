use std::collections::BTreeSet;
use std::slice;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing {
    Refused,
    Ignored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

#[derive(Clone, Debug)]
pub struct Migrator {
    migrations: Vec<Migration>,
    missing: Missing,
}

impl Migration {
    #[must_use]
    pub fn new(version: i64, description: &'static str, sql: &'static str) -> Migration {
        Migration { version, description, sql }
    }

    #[must_use]
    pub fn version(&self) -> i64 {
        self.version
    }

    pub fn rebuilt<Rebuilt>(&self, rebuild: impl FnOnce(i64, &'static str, &'static str) -> Rebuilt) -> Rebuilt {
        rebuild(self.version, self.description, self.sql)
    }
}

impl Migrator {
    pub fn iter(&self) -> slice::Iter<'_, Migration> {
        self.migrations.iter()
    }

    #[must_use]
    pub fn ignoring_missing(self) -> Migrator {
        Migrator { missing: Missing::Ignored, ..self }
    }

    pub fn undeclared(&self, applied: impl IntoIterator<Item = i64>) -> Option<i64> {
        let declared: BTreeSet<i64> = self.migrations.iter().map(Migration::version).collect();
        self.missing.refused(&declared, &applied.into_iter().collect())
    }
}

impl From<Vec<Migration>> for Migrator {
    fn from(migrations: Vec<Migration>) -> Migrator {
        Migrator { migrations, missing: Missing::Refused }
    }
}

impl<'migrator> IntoIterator for &'migrator Migrator {
    type Item = &'migrator Migration;
    type IntoIter = slice::Iter<'migrator, Migration>;

    fn into_iter(self) -> slice::Iter<'migrator, Migration> {
        self.migrations.iter()
    }
}

impl Missing {
    fn refused(self, declared: &BTreeSet<i64>, applied: &BTreeSet<i64>) -> Option<i64> {
        match self {
            Missing::Refused => applied.difference(declared).next().copied(),
            Missing::Ignored => None,
        }
    }
}
