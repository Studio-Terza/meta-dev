use std::collections::BTreeSet;
use std::slice;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

#[derive(Clone, Debug)]
pub struct Migrator {
    migrations: Vec<Migration>,
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

    pub fn undeclared(&self, applied: impl IntoIterator<Item = i64>) -> Option<i64> {
        let declared: BTreeSet<i64> = self.migrations.iter().map(Migration::version).collect();
        let seen: BTreeSet<i64> = applied.into_iter().collect();
        seen.difference(&declared).next().copied()
    }
}

impl From<Vec<Migration>> for Migrator {
    fn from(migrations: Vec<Migration>) -> Migrator {
        Migrator { migrations }
    }
}

impl<'migrator> IntoIterator for &'migrator Migrator {
    type Item = &'migrator Migration;
    type IntoIter = slice::Iter<'migrator, Migration>;

    fn into_iter(self) -> slice::Iter<'migrator, Migration> {
        self.migrations.iter()
    }
}
