use lint_no_ignored_tests::NoIgnoredTests;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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

const OTHER_ATTRIBUTE: &str = "#[test]\n#[serde(skip)]\nfn t() {}\n";

const CASES: [Case; 15] = [
    Case::new(
        Title::new("skipped"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(SKIPPED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("conditional ignore"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CONDITIONAL_IGNORE))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("conditional test"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CONDITIONAL_TEST))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("ignored law"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LAW))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("conditional law"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CONDITIONAL_LAW))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("ignored proptest"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PROPTEST))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("should panic runs"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(RUNS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("conditional helper"),
        &[Source::new(Located::new("src/a.rs"), Text::new(CONDITIONAL_HELPER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("derive helper"),
        &[Source::new(Located::new("src/a.rs"), Text::new(DERIVE_HELPER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("nested skip word"),
        &[Source::new(Located::new("src/a.rs"), Text::new(SERDE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("gated module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GATED_MODULE))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("gated file"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GATED_FILE))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("gated and test"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GATED_AND_TEST))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("test only module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TEST_ONLY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("skip inside another attribute"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(OTHER_ATTRIBUTE))],
        Flagged::new(&[]),
    ),
];

#[test]
fn skipped_tests_are_flagged() {
    assert_eq!(Sources::mismatches::<NoIgnoredTests>(&CASES), Lines::default());
}

const SWITCHED_OFF: Source =
    Source::new(Located::new("crates/a/Cargo.toml"), Text::new("[package]\nname = \"a\"\n\n[lib]\ntest = false\n"));
const SWITCHED_ON: Source = Source::new(Located::new("crates/a/Cargo.toml"), Text::new("[package]\nname = \"a\"\n"));
const GATED_LIBRARY: &[Source] = &[Source::new(
    Located::new("crates/a/src/lib.rs"),
    Text::new("pub fn f() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n"),
)];
const GATED_ITEM: &[Source] =
    &[Source::new(Located::new("crates/a/src/a.rs"), Text::new("#[cfg(all(test, unix))]\nfn helper() {}\n"))];
const GATED_FILE_IN_LIBRARY: &[Source] =
    &[Source::new(Located::new("crates/a/src/a.rs"), Text::new("#![cfg(test)]\nfn helper() {}\n"))];
const GATED_TESTS: &[Source] = &[Source::new(Located::new("crates/a/tests/a.rs"), Text::new(TEST_ONLY))];
const UNGATED: &[Source] = &[Source::new(Located::new("crates/a/src/lib.rs"), Text::new(CONDITIONAL_HELPER))];

struct Library(&'static [Source], &'static [Source]);

impl Library {
    fn flagged(&self) -> Lines {
        Sources::placed_flagged::<NoIgnoredTests>(self.0, self.1)
    }
}

laws::table!(
    test_code_in_a_library_whose_tests_are_switched_off_is_flagged_because_it_never_runs,
    Library::flagged,
    [
        Library(&[SWITCHED_OFF], GATED_LIBRARY) => ["crates/a/src/lib.rs:2"],
        Library(&[SWITCHED_OFF], GATED_ITEM) => ["crates/a/src/a.rs:1"],
        Library(&[SWITCHED_OFF], GATED_FILE_IN_LIBRARY) => ["crates/a/src/a.rs:1"],
        Library(&[SWITCHED_ON], GATED_LIBRARY) => Lines::default(),
        Library(&[SWITCHED_OFF], GATED_TESTS) => Lines::default(),
        Library(&[SWITCHED_OFF], UNGATED) => Lines::default(),
    ]
);
