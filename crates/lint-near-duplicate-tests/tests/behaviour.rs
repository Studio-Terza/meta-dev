use lint_near_duplicate_tests::NearDuplicateTests;

use lint_fixtures::{Case, Source, Sources};

const IDENTICAL: &str = "#[test]\nfn a() {\n    g();\n}\n#[test]\nfn b() {\n    g();\n}\n";
const NESTED: &str = "#[test]\nfn a() {\n    if h() {\n        f(1);\n    }\n}\n#[test]\nfn b() {\n    if h() {\n        f(2);\n    }\n}\n";
const INLINE_MODULES: &str = "mod m {\n    #[test]\n    fn a() {\n        f(1);\n    }\n}\nmod n {\n    #[test]\n    fn b() {\n        f(2);\n    }\n}\n";
const THREE: &str =
    "#[test]\nfn a() {\n    f(1);\n}\n#[test]\nfn b() {\n    f(2);\n}\n#[test]\nfn c() {\n    f(3);\n}\n";
const OTHER_CALL: &str = "#[test]\nfn a() {\n    f(1);\n}\n#[test]\nfn b() {\n    g(1);\n}\n";
const SHOULD_PANIC: &str = "#[test]\nfn a() {\n    f(1);\n}\n#[test]\n#[should_panic]\nfn b() {\n    f(2);\n}\n";
const HELPERS: &str = "fn a() {\n    f(1);\n}\nfn b() {\n    f(2);\n}\n";
const FIRST_FILE: &str = "#[test]\nfn a() {\n    f(1);\n}\n";
const SECOND_FILE: &str = "\n#[test]\nfn b() {\n    f(2);\n}\n";

const PANICS: &str = "#[test]\n#[should_panic(expected = \"big\")]\nfn a() {\n    f(10);\n}\n#[test]\n#[should_panic(expected = \"small\")]\nfn b() {\n    f(1);\n}\n";
const ASYNC: &str =
    "#[tokio::test]\nasync fn a() {\n    f(1).await;\n}\n#[tokio::test]\nasync fn b() {\n    f(2).await;\n}\n";
const SIGNED: &str =
    "#[test]\nfn a() {\n    assert_eq!(h(1), 1);\n}\n#[test]\nfn b() {\n    assert_eq!(h(-1), 1);\n}\n";

const GATED: &str = "#[test]\n#[cfg(unix)]\nfn a() {\n    f(1);\n}\n#[test]\n#[cfg(unix)]\nfn b() {\n    f(2);\n}\n";

const CASES: [Case; 13] = [
    Case::new("identical", &[Source::new("tests/a.rs", IDENTICAL)], &["tests/a.rs:6"]),
    Case::new("nested literals differ", &[Source::new("tests/a.rs", NESTED)], &["tests/a.rs:8"]),
    Case::new("across modules", &[Source::new("tests/a.rs", INLINE_MODULES)], &["tests/a.rs:9"]),
    Case::new("three alike", &[Source::new("tests/a.rs", THREE)], &["tests/a.rs:6", "tests/a.rs:10"]),
    Case::new("other call", &[Source::new("tests/a.rs", OTHER_CALL)], &[]),
    Case::new("should panic differs", &[Source::new("tests/a.rs", SHOULD_PANIC)], &[]),
    Case::new("both panic", &[Source::new("tests/a.rs", PANICS)], &[]),
    Case::new("async tests", &[Source::new("tests/a.rs", ASYNC)], &[]),
    Case::new("signed literal", &[Source::new("tests/a.rs", SIGNED)], &["tests/a.rs:6"]),
    Case::new("helpers are not tests", &[Source::new("tests/a.rs", HELPERS)], &[]),
    Case::new(
        "across files of a crate",
        &[Source::new("crates/a/tests/x.rs", FIRST_FILE), Source::new("crates/a/tests/y.rs", SECOND_FILE)],
        &["crates/a/tests/y.rs:3"],
    ),
    Case::new(
        "across crates",
        &[Source::new("crates/a/tests/x.rs", FIRST_FILE), Source::new("crates/b/tests/x.rs", SECOND_FILE)],
        &[],
    ),
    Case::new("gated alike", &[Source::new("tests/a.rs", GATED)], &["tests/a.rs:8"]),
];

#[test]
fn tests_differing_only_in_literals_are_flagged() {
    assert_eq!(Sources::mismatches::<NearDuplicateTests>(&CASES), Vec::<String>::new());
}
