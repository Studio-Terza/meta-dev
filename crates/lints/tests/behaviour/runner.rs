use std::error::Error;
use std::fs;

use crate::common::Sources;
use lint_core::Failure;
use lints::{Lints, Outcome};

use std::path::Path;

const DIRTY: &str = "// note\nfn render_page() {}\n";
const CLEAN: &str = "struct A;\n";
const LINT_COUNT: usize = 24;

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
    let outcome = Lints::check_root(root.path())?;
    let Outcome::Violations { reports, .. } = &outcome else {
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
    assert!(matches!(Lints::check_root(root.path()), Ok(Outcome::Clean)));
    Ok(())
}

struct Trees;

impl Trees {
    fn write(root: &Path, path: &str, text: &str) -> Result<(), Box<dyn Error>> {
        let file = root.join(path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, text)?;
        Ok(())
    }

    fn violation_count(outcome: &Outcome) -> usize {
        match outcome {
            Outcome::Clean => 0,
            Outcome::Violations { reports, .. } => reports.iter().map(|report| report.violations().len()).sum(),
        }
    }
}

#[test]
fn a_file_that_does_not_parse_is_reported_without_hiding_other_lints() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "// note\nmod broken;\n")?;
    Trees::write(root.path(), "src/broken.rs", "\n\nfn (")?;
    let outcome = Lints::check_root(root.path())?;
    let Outcome::Violations { failures, .. } = &outcome else {
        return Err("the tree has a failure".into());
    };
    assert!(matches!(failures.as_slice(), [Failure::Parse { .. }]));
    assert_eq!(Trees::violation_count(&outcome), 1);
    Ok(())
}

#[test]
fn a_parse_failure_names_its_line() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "mod broken;\n")?;
    Trees::write(root.path(), "src/broken.rs", "struct A;\nstruct B;\nfn f() { let x = (1; }\n")?;
    let Outcome::Violations { failures, .. } = Lints::check_root(root.path())? else {
        return Err("the tree has a failure".into());
    };
    let Some(Failure::Parse { line, .. }) = failures.first() else {
        return Err("a parse failure".into());
    };
    assert_eq!(line.to_string(), "3");
    Ok(())
}

#[test]
fn files_outside_every_crate_target_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", CLEAN)?;
    Trees::write(root.path(), "tests/ui/bad.rs", "fn (")?;
    Trees::write(root.path(), "tooling/corpus/dirty.rs", DIRTY)?;
    assert!(matches!(Lints::check_root(root.path()), Ok(Outcome::Clean)));
    Ok(())
}

#[test]
fn modules_named_like_build_directories_are_read() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "pub mod target;\n")?;
    Trees::write(root.path(), "src/target/mod.rs", DIRTY)?;
    assert_eq!(Trees::violation_count(&Lints::check_root(root.path())?), 2);
    Ok(())
}

#[test]
fn test_modules_in_their_own_files_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "#[cfg(test)]\nmod tests;\n")?;
    Trees::write(root.path(), "src/tests.rs", "struct S { on: bool }\n")?;
    assert!(matches!(Lints::check_root(root.path()), Ok(Outcome::Clean)));
    Ok(())
}

#[test]
fn a_declared_module_with_no_file_is_a_failure() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "mod absent;\n")?;
    let Outcome::Violations { failures, .. } = Lints::check_root(root.path())? else {
        return Err("the tree has a failure".into());
    };
    assert!(matches!(failures.as_slice(), [Failure::Io { .. }]));
    Ok(())
}

#[test]
fn a_tree_with_no_rust_sources_fails_closed() -> Result<(), Box<dyn Error>> {
    let empty = tempfile::tempdir()?;
    assert!(matches!(Lints::check_root(empty.path()), Err(Failure::NoSources { .. })));
    let other = tempfile::tempdir()?;
    Trees::write(other.path(), "README", "x")?;
    assert!(matches!(Lints::check_root(other.path()), Err(Failure::NoSources { .. })));
    Ok(())
}

#[test]
fn every_path_given_is_checked() -> Result<(), Box<dyn Error>> {
    let clean = tempfile::tempdir()?;
    Trees::write(clean.path(), "src/lib.rs", CLEAN)?;
    let dirty = tempfile::tempdir()?;
    Trees::write(dirty.path(), "src/lib.rs", DIRTY)?;
    let roots = [clean.path(), dirty.path()].map(Path::to_path_buf);
    assert!(matches!(Lints::check_all(&roots), Ok(Outcome::Violations { .. })));
    Ok(())
}

#[test]
fn the_binary_root_is_found_whichever_path_is_given() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/main.rs", "struct Extra;\nfn main() {}\n")?;
    assert_eq!(Trees::violation_count(&Lints::check_root(root.path())?), 1);
    assert_eq!(Trees::violation_count(&Lints::check_root(&root.path().join("src"))?), 1);
    Ok(())
}

#[test]
fn bin_directory_roots_are_checked() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/bin/tool.rs", "struct Extra;\nfn main() {}\n")?;
    Trees::write(root.path(), "src/bin/other/main.rs", "mod m {}\nfn main() {}\n")?;
    assert_eq!(Trees::violation_count(&Lints::check_root(root.path())?), 2);
    Ok(())
}

#[test]
fn symlink_loops_do_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", DIRTY)?;
    std::os::unix::fs::symlink(root.path(), root.path().join("src/loop"))?;
    assert_eq!(Trees::violation_count(&Lints::check_root(root.path())?), 2);
    Ok(())
}

#[test]
fn a_shebang_is_not_a_comment() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    Trees::write(root.path(), "src/lib.rs", "#!//usr/bin/env foo\nstruct A;\n")?;
    assert!(matches!(Lints::check_root(root.path()), Ok(Outcome::Clean)));
    Ok(())
}
