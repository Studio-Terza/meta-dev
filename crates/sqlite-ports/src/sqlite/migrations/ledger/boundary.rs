use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha384};
use store_ports::{Migration, Migrator, Version};

use super::super::super::failed::Unmigrated;

#[derive(Debug)]
pub(in super::super) struct Ledger {
    applied: BTreeMap<i64, Vec<u8>>,
}

impl Ledger {
    const TABLE: &'static str = "
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);";
    const DIRTY: &'static str = "SELECT version FROM _sqlx_migrations WHERE success = false ORDER BY version LIMIT 1";
    const APPLIED: &'static str = "SELECT version, checksum FROM _sqlx_migrations ORDER BY version";
    const RECORDED: &'static str = "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?1, ?2, TRUE, ?3, ?4)";

    pub(in super::super) fn read(connection: &Connection) -> Result<Ledger, Unmigrated> {
        connection.execute_batch(Self::TABLE)?;
        let dirty: Option<i64> = connection.query_row(Self::DIRTY, [], |row| row.get(0)).optional()?;
        dirty.map_or(Ok(()), |version| Err(Unmigrated::Dirty(version)))?;
        let mut statement = connection.prepare(Self::APPLIED)?;
        let applied = statement
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)))?
            .collect::<rusqlite::Result<BTreeMap<i64, Vec<u8>>>>()?;
        Ok(Ledger { applied })
    }

    pub(in super::super) fn applied(
        &self,
        declared: &Migrator,
        connection: &Connection,
    ) -> Result<BTreeSet<Version>, Unmigrated> {
        declared
            .undeclared(self.applied.keys().copied())
            .map_or(Ok(()), |version| Err(Unmigrated::Missing(version)))?;
        let mut ordered: Vec<&Migration> = declared.iter().collect();
        ordered.sort_by_key(|migration| migration.version());
        ordered.into_iter().filter_map(|migration| self.pending(migration, connection).transpose()).collect()
    }

    fn pending(&self, migration: &Migration, connection: &Connection) -> Result<Option<Version>, Unmigrated> {
        let step = migration.rebuilt(|version, description, sql| Step {
            version,
            description,
            sql,
            checksum: Sha384::digest(sql.as_bytes()).to_vec(),
        });
        self.applied.get(&step.version).map_or_else(|| step.applied(connection), |recorded| step.checked(recorded))
    }
}

#[derive(Debug)]
struct Step {
    version: i64,
    description: &'static str,
    sql: &'static str,
    checksum: Vec<u8>,
}

impl Step {
    fn checked(&self, recorded: &[u8]) -> Result<Option<Version>, Unmigrated> {
        (*recorded == *self.checksum).then_some(None).ok_or(Unmigrated::Mismatched(self.version))
    }

    fn applied(&self, connection: &Connection) -> Result<Option<Version>, Unmigrated> {
        let start = Instant::now();
        connection.execute_batch(self.sql)?;
        let elapsed = i64::try_from(start.elapsed().as_nanos()).unwrap_or(i64::MAX);
        connection.execute(Ledger::RECORDED, params![self.version, self.description, self.checksum, elapsed])?;
        Ok(Some(Version::new(self.version)))
    }
}
