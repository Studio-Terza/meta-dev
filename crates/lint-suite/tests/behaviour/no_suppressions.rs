use lint_core::{RepoFile, Report};
use lint_no_suppressions::NoSuppressions;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const ALLOWED: &str = "#[allow(dead_code)]\nfn f() {}\n";
const EXPECTED: &str = "#[expect(clippy::unwrap_used)]\nfn f() {}\n";
const INNER: &str = "#![allow(unused)]\nfn f() {}\n";
const CONDITIONAL: &str = "#[cfg_attr(test, allow(dead_code))]\nfn f() {}\n";
const NESTED: &str = "mod m {\n    impl A {\n        #[allow(unused)]\n        fn f() {}\n    }\n}\n";
const FIELD: &str = "struct A {\n    #[expect(dead_code)]\n    a: u8,\n}\n";
const OTHER: &str = "#[derive(Debug)]\n#[must_use]\nstruct A;\n";
const CONDITIONAL_OTHER: &str = "#[cfg_attr(test, derive(Debug))]\nstruct A;\n";
const QUOTED: &str = "const S: &str = \"#[allow(dead_code)]\";\n";
const PATHED: &str = "#[serde::allow]\nstruct A;\n";

const CASES: [Case; 10] = [
    Case::new(
        Title::new("allow"),
        &[Source::new(Located::new("src/a.rs"), Text::new(ALLOWED))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("expect"),
        &[Source::new(Located::new("src/a.rs"), Text::new(EXPECTED))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("inner"),
        &[Source::new(Located::new("src/a.rs"), Text::new(INNER))],
        Flagged::new(&["src/a.rs:1"]),
    ),
    Case::new(
        Title::new("conditional"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CONDITIONAL))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("nested"),
        &[Source::new(Located::new("src/a.rs"), Text::new(NESTED))],
        Flagged::new(&["src/a.rs:3"]),
    ),
    Case::new(
        Title::new("field"),
        &[Source::new(Located::new("src/a.rs"), Text::new(FIELD))],
        Flagged::new(&["src/a.rs:2"]),
    ),
    Case::new(Title::new("other"), &[Source::new(Located::new("src/a.rs"), Text::new(OTHER))], Flagged::new(&[])),
    Case::new(
        Title::new("conditional other"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CONDITIONAL_OTHER))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("quoted"), &[Source::new(Located::new("src/a.rs"), Text::new(QUOTED))], Flagged::new(&[])),
    Case::new(Title::new("pathed"), &[Source::new(Located::new("src/a.rs"), Text::new(PATHED))], Flagged::new(&[])),
];

#[test]
fn switched_off_lints_are_flagged() {
    assert_eq!(Sources::mismatches::<NoSuppressions>(&CASES), Lines::default());
}

#[test]
fn baseline_and_allow_list_files_are_flagged() {
    for (path, flagged) in [
        ("ci/lint-baseline.toml", true),
        ("self/Allowlist.txt", true),
        ("docs/allow-list.md", true),
        ("x/allow_list.json", true),
        ("y/suppressions.toml", true),
        ("z/IgnoreList", true),
        ("src/lib.rs", false),
        ("crates/lint/src/baseline.rs", false),
        ("ci/limits.toml", false),
        ("docs/allowed-words.md", false),
    ] {
        let report: Report = NoSuppressions::baselines(&[RepoFile::new(path, path)]);
        assert_eq!(Sources::lines(&report) != Lines::default(), flagged, "{path}");
    }
}
