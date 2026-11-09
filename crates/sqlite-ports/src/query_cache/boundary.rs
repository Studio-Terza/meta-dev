mod source;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::{Connection, Executor, SqlStr, Sqlite, SqliteConnection};
use sqlx_core::describe::Describe;
use tokio::runtime::{Builder, Runtime};

use source::Sources;

#[derive(Debug)]
pub struct QueryCache {
    root: PathBuf,
    migrations: PathBuf,
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
}

impl QueryCache {
    const DIRECTORY: &'static str = ".sqlx";
    const DATABASE: &'static str = "SQLite";
    const MEMORY: &'static str = "sqlite::memory:";

    pub fn prepared(&self, out: &mut impl io::Write) -> Result<(), Uncached> {
        let queries = Sources::under(&self.root)?.queries()?;
        let mut scratch = Scratch::migrated(&self.migrations)?;
        let kept = queries
            .iter()
            .map(|query| self.kept(&mut scratch, query, out))
            .collect::<Result<BTreeSet<PathBuf>, Uncached>>()?;
        self.pruned(&kept, out)
    }

    fn kept(&self, scratch: &mut Scratch, query: &str, out: &mut impl io::Write) -> Result<PathBuf, Uncached> {
        let hash = hex::encode(Sha256::digest(query.as_bytes()));
        let path = self.root.join(Self::DIRECTORY).join(format!("query-{hash}.json"));
        let text = scratch.described(query, hash)?;
        let current = fs::read(&path).ok();
        Some(text)
            .filter(|fresh| current.as_ref() != Some(fresh))
            .map_or_else(|| Ok(path.clone()), |fresh| self.rewritten(path.clone(), &fresh, out))
    }

    fn rewritten(&self, path: PathBuf, text: &[u8], out: &mut impl io::Write) -> Result<PathBuf, Uncached> {
        fs::write(&path, text).map_err(|failed| Uncached::Io { path: path.clone(), source: failed })?;
        self.announced("wrote", path, out)
    }

    fn announced(&self, change: &str, path: PathBuf, out: &mut impl io::Write) -> Result<PathBuf, Uncached> {
        let shown = path.strip_prefix(&self.root).unwrap_or(&path);
        writeln!(out, "{change} {}", shown.display())
            .map_err(|failed| Uncached::Io { path: path.clone(), source: failed })?;
        Ok(path)
    }

    fn pruned(&self, kept: &BTreeSet<PathBuf>, out: &mut impl io::Write) -> Result<(), Uncached> {
        let directory = self.root.join(Self::DIRECTORY);
        let unlisted = |failed| Uncached::Io { path: directory.clone(), source: failed };
        let listed = fs::read_dir(&directory)
            .map_err(unlisted)?
            .map(|entry| entry.map(|found| found.path()).map_err(unlisted))
            .collect::<Result<BTreeSet<PathBuf>, Uncached>>()?;
        listed.difference(kept).try_for_each(|stale| {
            fs::remove_file(stale).map_err(|failed| Uncached::Io { path: stale.clone(), source: failed })?;
            self.announced("removed", stale.clone(), out).map(|_removed| ())
        })
    }
}

#[derive(Debug)]
struct Scratch {
    runtime: Runtime,
    connection: SqliteConnection,
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
        Ok(Scratch { runtime, connection })
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
