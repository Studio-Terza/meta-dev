mod ignored;
mod source;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use console_ports::Answer;
use query_cache::Table;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::{Connection, Executor, SqlStr, Sqlite, SqliteConnection};
use sqlx_core::describe::Describe;
use tokio::runtime::{Builder, Runtime};

use crate::query_cache::boundary::ignored::Ignored;
use invariants::Ignoring;
use source::Sources;

#[derive(Debug)]
pub struct QueryCache {
    root: PathBuf,
    migrations: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    verb: &'static str,
    path: PathBuf,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes(Vec<Change>);

#[derive(Debug)]
struct Kept {
    path: PathBuf,
    change: Option<Change>,
}

#[derive(Serialize)]
struct Entry<'query> {
    db_name: &'static str,
    query: &'query str,
    describe: Describe<Sqlite>,
    hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Uncached {
    #[error("usage: describe_queries <workspace root> <migrations directory>")]
    Usage,
    #[error("cannot use {}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot read the workspace: {0}")]
    Unread(lint_layout::Failure),
    #[error("cannot start the runtime: {0}")]
    Runtime(io::Error),
    #[error("cannot open the scratch database: {0}")]
    Database(sqlx::Error),
    #[error("cannot migrate the scratch database: {0}")]
    Migrate(MigrateError),
    #[error("cannot describe {query:?}: {source}")]
    Undescribed { query: String, source: sqlx::Error },
    #[error("cannot write a cache entry: {0}")]
    Unwritten(serde_json::Error),
    #[error("cannot place a column: {0}")]
    Origin(query_cache::Unplaced),
}

impl QueryCache {
    const DIRECTORY: &'static str = ".sqlx";
    const DATABASE: &'static str = "SQLite";
    const MEMORY: &'static str = "sqlite::memory:";

    pub fn answered(arguments: Vec<OsString>) -> Answer {
        let prepared = QueryCache::try_from(arguments).and_then(|cache| cache.prepared());
        let shown = prepared.as_ref().ignoring(Ignored::Complained).map_or_default(ToString::to_string);
        Answer::reported(prepared).showing(shown)
    }

    pub fn prepared(&self) -> Result<Changes, Uncached> {
        let queries = Sources::queries(&self.root)?;
        let mut scratch = Scratch::migrated(&self.migrations)?;
        let tables = scratch.tables()?;
        let recorded: Vec<String> = tables.iter().flat_map(Table::statements).collect();
        let kept = queries
            .iter()
            .map(AsRef::as_ref)
            .chain(recorded.iter().map(String::as_str))
            .map(|query| self.kept(&mut scratch, query))
            .collect::<Result<Vec<Kept>, Uncached>>()?;
        let paths: BTreeSet<PathBuf> = kept.iter().map(|entry| entry.path.clone()).collect();
        let removed = self.pruned(&paths)?;
        Ok(Changes(kept.into_iter().filter_map(|entry| entry.change).chain(removed).collect()))
    }

    fn kept(&self, scratch: &mut Scratch, query: &str) -> Result<Kept, Uncached> {
        let hash = hex::encode(Sha256::digest(query.as_bytes()));
        let path = self.root.join(Self::DIRECTORY).join(format!("query-{hash}.json"));
        let text = scratch.described(query, hash)?;
        let current = fs::read(&path).ignoring(Ignored::Absent);
        let change = Some(text)
            .filter(|fresh| current.as_ref() != Some(fresh))
            .map(|fresh| self.rewritten(&path, &fresh))
            .transpose()?;
        Ok(Kept { path, change })
    }

    fn rewritten(&self, path: &Path, text: &[u8]) -> Result<Change, Uncached> {
        fs::write(path, text).map_err(|failed| Uncached::Io { path: path.to_path_buf(), source: failed })?;
        Ok(self.change(Change::WROTE, path))
    }

    fn change(&self, verb: &'static str, path: &Path) -> Change {
        Change { verb, path: path.strip_prefix(&self.root).ignoring(Ignored::Outside).unwrap_or(path).to_path_buf() }
    }

    fn pruned(&self, kept: &BTreeSet<PathBuf>) -> Result<Vec<Change>, Uncached> {
        let directory = self.root.join(Self::DIRECTORY);
        let unlisted = |failed| Uncached::Io { path: directory.clone(), source: failed };
        let listed = fs::read_dir(&directory)
            .map_err(unlisted)?
            .map(|entry| entry.map(|found| found.path()).map_err(unlisted))
            .collect::<Result<BTreeSet<PathBuf>, Uncached>>()?;
        listed
            .difference(kept)
            .map(|stale| {
                fs::remove_file(stale).map_err(|failed| Uncached::Io { path: stale.clone(), source: failed })?;
                Ok(self.change(Change::REMOVED, stale))
            })
            .collect()
    }
}

#[derive(Debug)]
struct Scratch {
    runtime: Runtime,
    connection: SqliteConnection,
    ledger: String,
}

impl TryFrom<Vec<OsString>> for QueryCache {
    type Error = Uncached;

    fn try_from(arguments: Vec<OsString>) -> Result<QueryCache, Uncached> {
        let [root, migrations] = <[OsString; 2]>::try_from(arguments).map_err(|_given| Uncached::Usage)?;
        Ok(QueryCache { root: PathBuf::from(root), migrations: PathBuf::from(migrations) })
    }
}

impl Scratch {
    fn migrated(migrations: &Path) -> Result<Scratch, Uncached> {
        let runtime = Builder::new_current_thread().enable_all().build().map_err(Uncached::Runtime)?;
        let mut connection =
            runtime.block_on(SqliteConnection::connect(QueryCache::MEMORY)).map_err(Uncached::Database)?;
        let migrator = runtime.block_on(Migrator::new(migrations)).map_err(Uncached::Migrate)?;
        runtime.block_on(migrator.run(&mut connection)).map_err(Uncached::Migrate)?;
        Ok(Scratch { runtime, connection, ledger: migrator.table_name.to_string() })
    }

    fn described(&mut self, query: &str, hash: String) -> Result<Vec<u8>, Uncached> {
        let sql = SqlStr::from_static(Box::leak(query.to_owned().into_boxed_str()));
        let describe = self
            .runtime
            .block_on(self.connection.describe(sql))
            .map_err(|failed| Uncached::Undescribed { query: query.to_owned(), source: failed })?;
        let entry = Entry { db_name: QueryCache::DATABASE, query, describe, hash };
        let mut text = serde_json::to_vec_pretty(&entry).map_err(Uncached::Unwritten)?;
        text.push(b'\n');
        Ok(text)
    }
}

impl Scratch {
    const RECORDS: &'static str = "SELECT t.name FROM pragma_table_list AS t WHERE t.schema = 'main' AND t.type = 'table' AND t.name != ?1 AND (SELECT count(*) FROM pragma_table_info(t.name) WHERE pk > 0) = 1 AND NOT EXISTS (SELECT 1 FROM pragma_table_info(t.name) WHERE pk = 0 AND \"notnull\" = 0) ORDER BY t.name";
    const COLUMNS: &'static str = "SELECT name, CASE WHEN pk > 0 AND upper(type) = 'INTEGER' AND NOT EXISTS (SELECT 1 FROM pragma_foreign_key_list(?1) AS f WHERE f.\"from\" = name) THEN 'generated_key' WHEN pk > 0 THEN 'key' WHEN dflt_value IS NOT NULL THEN 'generated' ELSE 'given' END FROM pragma_table_info(?1) ORDER BY cid";

    fn tables(&mut self) -> Result<Vec<Table>, Uncached> {
        let names: Vec<String> = self
            .runtime
            .block_on(sqlx::query_scalar(Self::RECORDS).bind(&self.ledger).fetch_all(&mut self.connection))
            .map_err(Uncached::Database)?;
        names.iter().map(|name| self.table(name)).collect()
    }

    fn table(&mut self, name: &str) -> Result<Table, Uncached> {
        let columns: Vec<(String, String)> = self
            .runtime
            .block_on(sqlx::query_as(Self::COLUMNS).bind(name).fetch_all(&mut self.connection))
            .map_err(Uncached::Database)?;
        columns.iter().try_fold(Table::named(name), |table, (column, origin)| {
            Ok(table.column(column, origin.parse().map_err(Uncached::Origin)?))
        })
    }
}

impl Change {
    const WROTE: &'static str = "wrote";
    const REMOVED: &'static str = "removed";
}

impl fmt::Display for Change {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.verb, self.path.display())
    }
}

impl fmt::Display for Changes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|change| writeln!(formatter, "{change}"))
    }
}
