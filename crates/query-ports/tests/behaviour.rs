use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use console_ports::{Complaint, Shown};
use golden::golden;
use query_cache::{Origin, Table};
use query_ports::QueryCache;
use sha2::{Digest, Sha256};
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
const RECORDS: &str = "CREATE TABLE r (id INTEGER PRIMARY KEY, name TEXT NOT NULL, at TEXT NOT NULL DEFAULT 'now');\nCREATE TABLE n (k TEXT PRIMARY KEY, v TEXT);\nCREATE TABLE p (a TEXT NOT NULL, b TEXT NOT NULL, PRIMARY KEY (a, b));\nCREATE TABLE k (id INTEGER PRIMARY KEY NOT NULL REFERENCES r (id), v TEXT NOT NULL);\n";
const WORKSPACE: &str = "[workspace]\nresolver = \"3\"\nmembers = [\"crates/*\"]\n";
const MANIFEST: &str = "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nstore-ports = { path = \"../store-ports\" }\n";
const PORT_MANIFEST: &str = "[package]\nname = \"store-ports\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nstore-ports-macros = { path = \"../store-ports-macros\" }\n";
const PORT: &str = "pub use store_ports_macros::query;\n";
const MACROS_MANIFEST: &str =
    "[package]\nname = \"store-ports-macros\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nproc-macro = true\n";
const MACROS: &str =
    "use proc_macro::TokenStream;\n\n#[proc_macro]\npub fn query(input: TokenStream) -> TokenStream {\n    input\n}\n";

struct Workspace(TempDir);

impl Workspace {
    fn laid() -> Result<Workspace, Box<dyn Error>> {
        let root = TempDir::new()?;
        let written = |relative: &str, text: &str| -> Result<(), Box<dyn Error>> {
            let path = root.path().join(relative);
            fs::create_dir_all(path.parent().ok_or("no parent")?)?;
            Ok(fs::write(path, text)?)
        };
        written("Cargo.toml", WORKSPACE)?;
        written("crates/a/Cargo.toml", MANIFEST)?;
        written("crates/store-ports/Cargo.toml", PORT_MANIFEST)?;
        written("crates/store-ports/src/lib.rs", PORT)?;
        written("crates/store-ports-macros/Cargo.toml", MACROS_MANIFEST)?;
        written("crates/store-ports-macros/src/lib.rs", MACROS)?;
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
    let answer = QueryCache::answered(arguments);
    assert_eq!(answer.code(), ExitCode::SUCCESS);
    let kept = Workspace::named(KEPT);
    assert_eq!(workspace.cached()?, BTreeSet::from([kept.clone()]));
    let entry = fs::read_to_string(workspace.path().join(".sqlx").join(&kept))?;
    assert!(entry.contains(r#""query": "SELECT 1 AS one""#));
    assert!(entry.contains(r#""name": "one""#));
    golden!("prepared.txt", AsRef::<str>::as_ref(AsRef::<Shown>::as_ref(&answer)));
    Ok(())
}

#[test]
fn a_run_given_the_wrong_arguments_exits_with_failure_and_prints_its_usage_to_standard_error() {
    let answer = QueryCache::answered(vec![OsString::from("only-a-root")]);
    assert_eq!(answer.code(), ExitCode::FAILURE);
    assert_eq!(AsRef::<str>::as_ref(AsRef::<Shown>::as_ref(&answer)), "");
    golden!("usage.txt", AsRef::<str>::as_ref(AsRef::<Complaint>::as_ref(&answer)));
}

#[test]
fn a_second_run_over_a_cache_that_matches_its_queries_changes_nothing() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::laid()?;
    let arguments = || vec![OsString::from(workspace.path()), OsString::from(workspace.path().join("migrations"))];
    QueryCache::try_from(arguments())?.prepared()?;
    assert_eq!(QueryCache::try_from(arguments())?.prepared()?.to_string(), "");
    Ok(())
}

#[test]
fn a_table_with_one_key_and_no_nullable_columns_gets_its_record_statements() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::laid()?;
    fs::write(workspace.path().join("migrations/20260102000000_records.sql"), RECORDS)?;
    let arguments = vec![OsString::from(workspace.path()), OsString::from(workspace.path().join("migrations"))];
    QueryCache::try_from(arguments)?.prepared()?;
    let record = Table::named("r")
        .column("id", Origin::GeneratedKey)
        .column("name", Origin::Given)
        .column("at", Origin::Generated);
    let referencing = Table::named("k").column("id", Origin::Key).column("v", Origin::Given);
    let expected = [record, referencing]
        .iter()
        .flat_map(Table::statements)
        .map(|sql| Workspace::named(&sql))
        .chain([Workspace::named(KEPT)])
        .collect();
    assert_eq!(workspace.cached()?, expected);
    Ok(())
}
