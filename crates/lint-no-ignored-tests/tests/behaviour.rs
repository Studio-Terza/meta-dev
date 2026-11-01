use std::error::Error;
use std::fs;
use std::path::Path;

use lint_core::{Lint, SourceFiles};
use lint_no_ignored_tests::NoIgnoredTests;

use lint_fixtures::{Case, Source, Sources};

const IGNORED: &str = "#[test]\n#[ignore]\nfn t() {}\n";
const REASON: &str = "#[test]\n#[ignore = \"slow\"]\nfn t() {}\n";
const SKIPPED: &str = "#[rstest]\n#[skip]\nfn t() {}\n";
const CONDITIONAL_IGNORE: &str = "#[test]\n#[cfg_attr(miri, ignore)]\nfn t() {}\n";
const CONDITIONAL_TEST: &str = "#[test]\n#[cfg(unix)]\nfn t() {}\n";
const LAW: &str = "laws::table!(\n    #[ignore]\n    t,\n    f,\n    [1 => 2]\n);\n";
const CONDITIONAL_LAW: &str = "laws::table!(\n    #[cfg(unix)]\n    t,\n    f,\n    [1 => 2]\n);\n";
const PROPTEST: &str = "proptest! {\n    #[test]\n    #[ignore]\n    fn t(a in 0..2) {}\n}\n";
const RUNS: &str = "#[test]\n#[should_panic]\nfn t() {}\n";
const CONDITIONAL_HELPER: &str = "#[cfg(unix)]\nfn f() {}\n";
const DERIVE_HELPER: &str = "#[derive(Debug)]\nstruct A {\n    #[skip]\n    a: u8,\n}\n";
const SERDE: &str = "#[cfg_attr(feature = \"serde\", serde(skip))]\nfn f() {}\n";

const GATED_MODULE: &str = "#[cfg(windows)]\nmod m {\n    #[test]\n    fn t() {}\n}\n";
const GATED_FILE: &str = "#![cfg(windows)]\n#[test]\nfn t() {}\n";
const GATED_AND_TEST: &str = "#[test]\n#[cfg(all(test, windows))]\nfn t() {}\n";
const TEST_ONLY: &str = "#[cfg(test)]\nmod m {\n    #[test]\n    fn t() {}\n}\n";

const CASES: [Case; 16] = [
    Case::new("ignored", &[Source::new("tests/a.rs", IGNORED)], &["tests/a.rs:2"]),
    Case::new("ignored with a reason", &[Source::new("tests/a.rs", REASON)], &["tests/a.rs:2"]),
    Case::new("skipped", &[Source::new("tests/a.rs", SKIPPED)], &["tests/a.rs:2"]),
    Case::new("conditional ignore", &[Source::new("tests/a.rs", CONDITIONAL_IGNORE)], &["tests/a.rs:2"]),
    Case::new("conditional test", &[Source::new("tests/a.rs", CONDITIONAL_TEST)], &["tests/a.rs:2"]),
    Case::new("ignored law", &[Source::new("tests/a.rs", LAW)], &["tests/a.rs:2"]),
    Case::new("conditional law", &[Source::new("tests/a.rs", CONDITIONAL_LAW)], &["tests/a.rs:2"]),
    Case::new("ignored proptest", &[Source::new("tests/a.rs", PROPTEST)], &["tests/a.rs:3"]),
    Case::new("should panic runs", &[Source::new("tests/a.rs", RUNS)], &[]),
    Case::new("conditional helper", &[Source::new("src/a.rs", CONDITIONAL_HELPER)], &[]),
    Case::new("derive helper", &[Source::new("src/a.rs", DERIVE_HELPER)], &[]),
    Case::new("nested skip word", &[Source::new("src/a.rs", SERDE)], &[]),
    Case::new("gated module", &[Source::new("tests/a.rs", GATED_MODULE)], &["tests/a.rs:1"]),
    Case::new("gated file", &[Source::new("tests/a.rs", GATED_FILE)], &["tests/a.rs:1"]),
    Case::new("gated and test", &[Source::new("tests/a.rs", GATED_AND_TEST)], &["tests/a.rs:2"]),
    Case::new("test only module", &[Source::new("tests/a.rs", TEST_ONLY)], &[]),
];

#[test]
fn skipped_tests_are_flagged() {
    assert_eq!(Sources::mismatches::<NoIgnoredTests>(&CASES), Vec::<String>::new());
}

#[test]
fn the_message_names_the_fix() {
    let report = NoIgnoredTests::report(&[Sources::file("tests/a.rs", IGNORED)]);
    assert!(report.violations().iter().all(|violation| violation.message().contains("remove the attribute")));
    assert!(!report.is_clean());
}

#[test]
fn a_test_target_switched_off_in_the_manifest_is_flagged() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let manifest = "[package]\nname = \"a\"\n[[test]]\nname = \"off\"\npath = \"tests/off.rs\"\ntest = false\n";
    fs::write(root.path().join("Cargo.toml"), manifest)?;
    fs::create_dir_all(root.path().join("src"))?;
    fs::create_dir_all(root.path().join("tests"))?;
    fs::write(root.path().join("src/lib.rs"), "")?;
    fs::write(root.path().join("tests/off.rs"), "#[test]\nfn t() {}\n")?;
    fs::write(root.path().join("tests/on.rs"), "#[test]\nfn t() {}\n")?;
    let report = NoIgnoredTests::report(SourceFiles::read(root.path())?.files());
    let sites: Vec<&Path> = report.violations().iter().map(|violation| violation.site().path()).collect();
    assert!(matches!(sites.as_slice(), [only] if only.ends_with("tests/off.rs")));
    Ok(())
}
