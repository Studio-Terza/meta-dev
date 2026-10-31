mod common;

use std::error::Error;
use std::fs;

use common::Sources;
use lints::{Lints, Outcome};

const DIRTY: &str = "// note\nfn helper() {}\n";
const CLEAN: &str = "struct A;\n";
const LINT_COUNT: usize = 8;

#[test]
fn every_lint_reports_over_the_same_files() {
    let reports = Lints::reports(&[Sources::file("a.rs", DIRTY)]);
    assert_eq!(reports.len(), LINT_COUNT);
    let dirty = reports.iter().filter(|report| !report.is_clean()).count();
    assert_eq!(dirty, 2);
    let lines: Vec<String> = reports.iter().flat_map(Sources::lines).collect();
    assert_eq!(lines, ["a.rs:1", "a.rs:2"]);
}

#[test]
fn a_tree_with_violations_is_not_clean() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("src"))?;
    fs::write(root.path().join("src/lib.rs"), DIRTY)?;
    fs::write(root.path().join("src/clean.rs"), CLEAN)?;
    let outcome = Lints::check(root.path())?;
    let Outcome::Violations(reports) = &outcome else {
        return Err("the tree has violations".into());
    };
    assert_eq!(reports.len(), 2);
    assert!(outcome.to_string().contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn a_clean_tree_is_clean_and_build_directories_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("src"))?;
    fs::create_dir_all(root.path().join("target"))?;
    fs::write(root.path().join("src/lib.rs"), CLEAN)?;
    fs::write(root.path().join("target/out.rs"), DIRTY)?;
    assert!(matches!(Lints::check(root.path()), Ok(Outcome::Clean)));
    Ok(())
}

#[test]
fn a_file_that_does_not_parse_is_a_failure() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("broken.rs"), "fn (")?;
    assert!(Lints::check(root.path()).is_err());
    Ok(())
}
