use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use golden::golden;
use sha2::{Digest, Sha256};
use sqlite_ports::QueryCache;
use tempfile::TempDir;

const KEPT: &str = "SELECT 1 AS one";
const SOURCE: &str = r#"
fn kept() {
    store_ports::query!(Row, "SELECT 1 AS one", id: Id);
    other::query!("SELECT 2 AS two");
}
"#;
const FIXTURE: &str = r#"fn fixture() { store_ports::query!("SELECT 3 AS three"); }"#;
const NOTES: &str = r#"store_ports::query!("SELECT 4 AS four")"#;
const STALE: &str = "query-stale.json";

struct Workspace(TempDir);

impl Workspace {
    fn laid() -> Result<Workspace, Box<dyn Error>> {
        let root = TempDir::new()?;
        let written = |relative: &str, text: &str| -> Result<(), Box<dyn Error>> {
            let path = root.path().join(relative);
            fs::create_dir_all(path.parent().ok_or("no parent")?)?;
            Ok(fs::write(path, text)?)
        };
        written("crates/a/src/lib.rs", SOURCE)?;
        written("crates/a/tests/fixtures/src/lib.rs", FIXTURE)?;
        written("crates/a/notes.txt", NOTES)?;
        written(&format!(".sqlx/{STALE}"), "{}")?;
        written("migrations/20260101000000_tables.sql", "CREATE TABLE t (a INTEGER);")?;
        Ok(Workspace(root))
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn cached(&self) -> Result<BTreeSet<String>, Box<dyn Error>> {
        fs::read_dir(self.path().join(".sqlx"))?
            .map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned()))
            .collect()
    }

    fn named(query: &str) -> String {
        format!("query-{}.json", hex::encode(Sha256::digest(query.as_bytes())))
    }
}

#[test]
fn every_store_query_in_the_crates_gets_an_entry_and_entries_no_query_uses_are_removed() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::laid()?;
    let arguments = vec![OsString::from(workspace.path()), OsString::from(workspace.path().join("migrations"))];
    let mut out = Vec::new();
    QueryCache::try_from(arguments)?.prepared(&mut out)?;
    let kept = Workspace::named(KEPT);
    assert_eq!(workspace.cached()?, BTreeSet::from([kept.clone()]));
    let entry = fs::read_to_string(workspace.path().join(".sqlx").join(&kept))?;
    assert!(entry.contains(r#""query": "SELECT 1 AS one""#));
    assert!(entry.contains(r#""name": "one""#));
    let printed = String::from_utf8(out)?;
    golden!("prepared.txt", &printed);
    Ok(())
}

#[test]
fn a_second_run_over_a_cache_that_matches_its_queries_changes_nothing() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::laid()?;
    let arguments = || vec![OsString::from(workspace.path()), OsString::from(workspace.path().join("migrations"))];
    QueryCache::try_from(arguments())?.prepared(&mut Vec::new())?;
    let mut out = Vec::new();
    QueryCache::try_from(arguments())?.prepared(&mut out)?;
    assert_eq!(out, Vec::<u8>::new());
    Ok(())
}
