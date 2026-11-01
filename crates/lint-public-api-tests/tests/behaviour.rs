use lint_core::Lint;
use lint_public_api_tests::PublicApiTests;

use lint_fixtures::{Case, Source, Sources};

const MODULE: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
const MODULE_FILE: &str = "#[cfg(test)]\nmod tests;\n";
const COMBINED: &str = "#[cfg(all(test, unix))]\nmod tests {}\n";
const NOT_TEST: &str = "#[cfg(not(test))]\nmod real {}\n";
const BARE_TEST: &str = "#[test]\nfn t() {}\n";
const NESTED: &str = "mod inner {\n    #[test]\n    fn t() {}\n}\n";
const LAW: &str = "laws::table!(t, f, [1 => 2]);\n";
const DEFINED: &str = "macro_rules! m {\n    () => {\n        #[test]\n        fn t() {}\n    };\n}\n";
const PLAIN: &str = "mod inner {\n    fn f() {}\n}\n";

const CASES: [Case; 11] = [
    Case::new("inline module", &[Source::new("src/a.rs", MODULE)], &["src/a.rs:3"]),
    Case::new("module file", &[Source::new("src/a.rs", MODULE_FILE)], &["src/a.rs:2"]),
    Case::new("combined condition", &[Source::new("src/a.rs", COMBINED)], &["src/a.rs:2"]),
    Case::new("not test", &[Source::new("src/a.rs", NOT_TEST)], &[]),
    Case::new("bare test", &[Source::new("src/a.rs", BARE_TEST)], &["src/a.rs:2"]),
    Case::new("nested test", &[Source::new("src/a.rs", NESTED)], &["src/a.rs:3"]),
    Case::new("law in src", &[Source::new("src/a.rs", LAW)], &["src/a.rs:1"]),
    Case::new("macro definition", &[Source::new("src/a.rs", DEFINED)], &[]),
    Case::new("plain module", &[Source::new("src/a.rs", PLAIN)], &[]),
    Case::new("test in tests", &[Source::new("tests/a.rs", BARE_TEST)], &[]),
    Case::new("law in tests", &[Source::new("tests/a.rs", LAW)], &[]),
];

#[test]
fn test_code_in_src_is_flagged() {
    assert_eq!(Sources::mismatches::<PublicApiTests>(&CASES), Vec::<String>::new());
}

#[test]
fn the_message_names_the_fix() {
    let report = PublicApiTests::report(&[Sources::file("src/a.rs", MODULE)]);
    assert!(report.violations().iter().all(|violation| violation.message().contains("tests directory")));
    assert!(!report.is_clean());
}
