use std::error::Error;
use std::fs;

use lint_core::Lint;
use lint_core::Violation;
use lint_no_test_only_api::NoTestOnlyApi;
use lint_reader::SourceFiles;

use lint_fixtures::{Case, Source, Sources};

const GATED_FN: &str = "#[cfg(test)]\npub fn f() {}\n";
const GATED_CRATE_FN: &str = "#[cfg(test)]\npub(crate) fn f() {}\n";
const GATED_ANY: &str = "#[cfg(any(test, unix))]\npub struct A;\n";
const GATED_METHOD: &str = "struct A;\nimpl A {\n    #[cfg(test)]\n    pub fn f() {}\n}\n";
const GATED_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    pub fn f() {}\n}\n";
const GATED_FIELD: &str = "struct A {\n    #[cfg(test)]\n    pub a: u8,\n}\n";
const GATED_PRIVATE: &str = "#[cfg(test)]\nfn f() {}\n";
const NOT_TEST: &str = "#[cfg(not(test))]\npub fn f() {}\n";
const HIDDEN: &str = "#[doc(hidden)]\npub fn f() {}\n";
const HIDDEN_METHOD: &str = "struct A;\nimpl A {\n    #[doc(hidden)]\n    pub fn f() {}\n}\n";
const HIDDEN_PRIVATE: &str = "#[doc(hidden)]\nfn f() {}\n";
const DOCUMENTED: &str = "#[doc = \"hidden\"]\npub fn f() {}\n";
const FEATURE: &str = "#[cfg(feature = \"test-utils\")]\nfn f() {}\n";
const ANY_FEATURE: &str = "#[cfg(any(unix, feature = \"mocks\"))]\nfn f() {}\n";
const FEATURE_ATTRIBUTE: &str = "#[cfg_attr(feature = \"testing\", derive(Debug))]\nstruct A;\n";
const FEATURE_MACRO: &str = "fn f() -> bool {\n    cfg!(feature = \"test_helpers\")\n}\n";
const OTHER_FEATURE: &str = "#[cfg(feature = \"latest\")]\nfn f() {}\n#[cfg(feature = \"contest\")]\nfn g() {}\n";

const CASES: [Case; 19] = [
    Case::new("gated fn", &[Source::new("src/a.rs", GATED_FN)], &["src/a.rs:2"]),
    Case::new("gated crate fn", &[Source::new("src/a.rs", GATED_CRATE_FN)], &["src/a.rs:2"]),
    Case::new("gated any ships elsewhere", &[Source::new("src/a.rs", GATED_ANY)], &[]),
    Case::new("gated method", &[Source::new("src/a.rs", GATED_METHOD)], &["src/a.rs:4"]),
    Case::new("gated impl", &[Source::new("src/a.rs", GATED_IMPL)], &["src/a.rs:4"]),
    Case::new("gated field", &[Source::new("src/a.rs", GATED_FIELD)], &["src/a.rs:3"]),
    Case::new("gated private", &[Source::new("src/a.rs", GATED_PRIVATE)], &[]),
    Case::new("not test", &[Source::new("src/a.rs", NOT_TEST)], &[]),
    Case::new("hidden", &[Source::new("src/a.rs", HIDDEN)], &["src/a.rs:2"]),
    Case::new("hidden method", &[Source::new("src/a.rs", HIDDEN_METHOD)], &["src/a.rs:4"]),
    Case::new("hidden private", &[Source::new("src/a.rs", HIDDEN_PRIVATE)], &[]),
    Case::new("doc string", &[Source::new("src/a.rs", DOCUMENTED)], &[]),
    Case::new("gated in tests", &[Source::new("tests/a.rs", GATED_FN)], &[]),
    Case::new("feature", &[Source::new("src/a.rs", FEATURE)], &["src/a.rs:1"]),
    Case::new("feature in any", &[Source::new("src/a.rs", ANY_FEATURE)], &["src/a.rs:1"]),
    Case::new("feature in cfg_attr", &[Source::new("src/a.rs", FEATURE_ATTRIBUTE)], &["src/a.rs:1"]),
    Case::new("feature in cfg macro", &[Source::new("src/a.rs", FEATURE_MACRO)], &["src/a.rs:2"]),
    Case::new("feature in tests", &[Source::new("tests/a.rs", FEATURE)], &["tests/a.rs:1"]),
    Case::new("other feature", &[Source::new("src/a.rs", OTHER_FEATURE)], &[]),
];

#[test]
fn test_only_api_is_flagged() {
    assert_eq!(Sources::mismatches::<NoTestOnlyApi>(&CASES), Vec::<String>::new());
}

#[test]
fn a_test_feature_in_the_manifest_is_flagged_once() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let manifest = "[package]\nname = \"a\"\n[features]\ntest-utils = []\nfast = []\n";
    fs::write(root.path().join("Cargo.toml"), manifest)?;
    fs::create_dir(root.path().join("src"))?;
    fs::write(root.path().join("src/lib.rs"), "mod other;\n")?;
    fs::write(root.path().join("src/other.rs"), "")?;
    let report = NoTestOnlyApi::report(SourceFiles::read(root.path())?.files());
    let messages: Vec<&str> = report.violations().iter().map(Violation::message).collect();
    assert!(
        matches!(messages.as_slice(), [only] if only.contains("test-utils") && only.contains("remove the feature"))
    );
    Ok(())
}

#[test]
fn the_messages_name_the_fix() {
    let report = NoTestOnlyApi::report(&[Sources::file("src/a.rs", GATED_FN), Sources::file("src/b.rs", HIDDEN)]);
    assert!(report.violations().iter().all(|violation| violation.message().contains("; ")));
    assert_eq!(report.violations().len(), 2);
}
