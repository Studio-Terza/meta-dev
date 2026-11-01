use lint_near_duplicate_tests::NearDuplicateTests;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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
    Case::new(
        Title::new("identical"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(IDENTICAL))],
        Flagged::new(&["tests/a.rs:6"]),
    ),
    Case::new(
        Title::new("nested literals differ"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(NESTED))],
        Flagged::new(&["tests/a.rs:8"]),
    ),
    Case::new(
        Title::new("across modules"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(INLINE_MODULES))],
        Flagged::new(&["tests/a.rs:9"]),
    ),
    Case::new(
        Title::new("three alike"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(THREE))],
        Flagged::new(&["tests/a.rs:6", "tests/a.rs:10"]),
    ),
    Case::new(
        Title::new("other call"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(OTHER_CALL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("should panic differs"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(SHOULD_PANIC))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("both panic"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PANICS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("async tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ASYNC))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("signed literal"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(SIGNED))],
        Flagged::new(&["tests/a.rs:6"]),
    ),
    Case::new(
        Title::new("helpers are not tests"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(HELPERS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("across files of a crate"),
        &[
            Source::new(Located::new("crates/a/tests/x.rs"), Text::new(FIRST_FILE)),
            Source::new(Located::new("crates/a/tests/y.rs"), Text::new(SECOND_FILE)),
        ],
        Flagged::new(&["crates/a/tests/y.rs:3"]),
    ),
    Case::new(
        Title::new("across crates"),
        &[
            Source::new(Located::new("crates/a/tests/x.rs"), Text::new(FIRST_FILE)),
            Source::new(Located::new("crates/b/tests/x.rs"), Text::new(SECOND_FILE)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("gated alike"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GATED))],
        Flagged::new(&["tests/a.rs:8"]),
    ),
];

#[test]
fn tests_differing_only_in_literals_are_flagged() {
    assert_eq!(Sources::mismatches::<NearDuplicateTests>(&CASES), Lines::default());
}
