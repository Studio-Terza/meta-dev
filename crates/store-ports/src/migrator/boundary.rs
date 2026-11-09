use std::slice;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transacted {
    Wrapped,
    Unwrapped,
}

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
    transacted: Transacted,
}

#[derive(Clone, Debug)]
pub struct Migrator {
    migrations: Vec<Migration>,
    missing: Missing,
}

impl From<Transacted> for bool {
    fn from(transacted: Transacted) -> bool {
        match transacted {
            Transacted::Wrapped => false,
            Transacted::Unwrapped => true,
        }
    }
}

impl From<Missing> for bool {
    fn from(missing: Missing) -> bool {
        match missing {
            Missing::Refused => false,
            Missing::Ignored => true,
        }
    }
}

impl Migration {
    #[must_use]
    pub fn new(version: i64, description: &'static str, sql: &'static str, transacted: Transacted) -> Migration {
        Migration { version, description, sql, transacted }
    }

    #[must_use]
    pub fn version(&self) -> i64 {
        self.version
    }

    pub fn rebuilt<Rebuilt>(
        &self,
        rebuild: impl FnOnce(i64, &'static str, &'static str, Transacted) -> Rebuilt,
    ) -> Rebuilt {
        rebuild(self.version, self.description, self.sql, self.transacted)
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

    #[must_use]
    pub fn missing(&self) -> Missing {
        self.missing
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
