use lint_public_api_tests::PublicApiTests;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const MODULE: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
const MODULE_FILE: &str = "#[cfg(test)]\nmod tests;\n";
const COMBINED: &str = "#[cfg(all(test, unix))]\nmod tests {}\n";
const NOT_TEST: &str = "#[cfg(not(test))]\nmod real {}\n";
const BARE_TEST: &str = "#[test]\nfn t() {}\n";
const NESTED: &str = "mod inner {\n    #[test]\n    fn t() {}\n}\n";
const LAW: &str = "laws::table!(t, f, [1 => 2]);\n";
const DEFINED: &str = "macro_rules! m {\n    () => {\n        #[test]\n        fn t() {}\n    };\n}\n";
const PLAIN: &str = "mod inner {\n    fn f() {}\n}\n";

const PROPTEST: &str = "proptest::proptest! {\n    #[test]\n    fn t(a in 0..2) {}\n}\n";
const EMITTER: &str = "macro_rules! check {\n    ($name:ident) => {\n        #[test]\n        fn $name() {}\n    };\n}\ncheck!(private_internals);\n";
const RSTEST: &str = "#[rstest::rstest]\nfn t() {}\n";
const SCHEMA: &str = "diesel::table! {\n    users (id) {\n        id -> Int4,\n    }\n}\n";
const PEEK: &str = "#[path = \"../src/internal.rs\"]\nmod internal;\n";
const NESTED_PEEK: &str = "mod m {\n    #[path = \"../../src/internal.rs\"]\n    mod internal;\n}\n";
const SIBLING: &str = "#[path = \"common/mod.rs\"]\nmod common;\n";

const PLAIN_MACRO: &str = "macro_rules! plain {\n    () => {};\n}\nplain!();\n";
const OUTSIDE_THE_PACKAGE: &str = "#[path = \"../../src/internal.rs\"]\nmod internal;\n";

const FEATURE_MODULE: &str = "#[cfg(any(test, feature = \"proptest\"))]\nmod support {\n    pub fn f() {}\n}\n";
const ANY_TESTS: &str = "#[cfg(any(test, all(test, unix)))]\nmod tests {}\n";

const NEVER: &str = "#[cfg(any())]\nmod never {}\n";

const CASES: [Case; 23] = [
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(MODULE))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("module file"),
        &[Source::new(Located::new("src/a.rs"), Text::new(MODULE_FILE))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("combined condition"),
        &[Source::new(Located::new("src/a.rs"), Text::new(COMBINED))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("test or feature ships"),
        &[Source::new(Located::new("src/a.rs"), Text::new(FEATURE_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("every alternative is test only"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ANY_TESTS))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(Title::new("empty any"), &[Source::new(Located::new("src/a.rs"), Text::new(NEVER))], Flagged::new(&[])),
    Case::new(Title::new("not test"), &[Source::new(Located::new("src/a.rs"), Text::new(NOT_TEST))], Flagged::new(&[])),
    Case::new(
        Title::new("bare test"),
        &[Source::new(Located::new("src/a.rs"), Text::new(BARE_TEST))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("nested test"),
        &[Source::new(Located::new("src/a.rs"), Text::new(NESTED))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("law in src"),
        &[Source::new(Located::new("src/a.rs"), Text::new(LAW))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("macro definition"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DEFINED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("plain module"),
        &[Source::new(Located::new("src/a.rs"), Text::new(PLAIN))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test in tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(BARE_TEST))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("law in tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LAW))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("proptest block"),
        &[Source::new(Located::new("src/a.rs"), Text::new(PROPTEST))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("macro that emits a test"),
        &[Source::new(Located::new("src/a.rs"), Text::new(EMITTER))],
        Flagged::new(&["src/a.rs:7"]),
    ),
    Case::new(
        Title::new("rstest"),
        &[Source::new(Located::new("src/a.rs"), Text::new(RSTEST))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(
        Title::new("foreign table macro"),
        &[Source::new(Located::new("src/schema.rs"), Text::new(SCHEMA))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("path into src"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PEEK))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("path into src from a module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(NESTED_PEEK))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("path to a helper"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(SIBLING))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("macro with no test"),
        &[Source::new(Located::new("src/a.rs"), Text::new(PLAIN_MACRO))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("path outside the package"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(OUTSIDE_THE_PACKAGE))],
        Flagged::new(&[]),
    ),
];

#[test]
fn test_code_in_src_is_flagged() {
    assert_eq!(Sources::mismatches::<PublicApiTests>(&CASES), Lines::default());
}
