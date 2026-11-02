use lint_public_surface::PublicSurface;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const EXPORTER: Source =
    Source::new(Located::new("crates/a/src/lib.rs"), Text::new("mod inner;\npub use inner::{Used, Unused};\n"));
const USER: Source = Source::new(Located::new("crates/b/src/lib.rs"), Text::new("use a::Used;\n"));
const PATH_USER: Source =
    Source::new(Located::new("crates/b/src/lib.rs"), Text::new("fn f(x: a::Used, y: a::Unused) {}\n"));
const SELF_USER: Source = Source::new(Located::new("crates/a/src/inner.rs"), Text::new("use a::Unused;\n"));
const BINARY: Source = Source::new(Located::new("crates/a/src/main.rs"), Text::new("use a::Unused;\n"));
const LONELY: Source =
    Source::new(Located::new("crates/c/src/lib.rs"), Text::new("mod inner;\npub use inner::Alone;\n"));

const MACRO_USER: Source = Source::new(
    Located::new("crates/b/src/lib.rs"),
    Text::new("use a::Used;\nfn f() {\n    println!(\"{:?}\", a::Unused);\n}\n"),
);
const GLOB_USER: Source = Source::new(Located::new("crates/b/src/lib.rs"), Text::new("use a::*;\n"));
const CRATE_ONLY: Source = Source::new(
    Located::new("crates/a/src/lib.rs"),
    Text::new("mod inner;\npub use inner::Used;\npub(crate) use inner::Hidden;\n"),
);

const OTHER_GLOB: Source = Source::new(Located::new("crates/c/src/lib.rs"), Text::new("use a::*;\n"));
const SELF_GLOB: Source = Source::new(Located::new("crates/a/src/inner.rs"), Text::new("use a::*;\n"));

const TEST_USER: Source = Source::new(Located::new("crates/a/tests/api.rs"), Text::new("use a::Unused;\n"));
const TEST_USER_ALL: Source = Source::new(Located::new("crates/a/tests/api.rs"), Text::new("use a::{Used, Unused};\n"));

const MODULES: Source = Source::new(
    Located::new("crates/a/src/lib.rs"),
    Text::new("pub mod inner;\npub mod used;\npub(crate) mod hidden;\n"),
);
const MODULE_USER: Source = Source::new(Located::new("crates/b/src/lib.rs"), Text::new("use a::used::Thing;\n"));
const MODULE_PATH_USER: Source =
    Source::new(Located::new("crates/b/src/lib.rs"), Text::new("fn f() {\n    a::used::go();\n}\n"));
const MODULE_GROUP_USER: Source =
    Source::new(Located::new("crates/b/src/lib.rs"), Text::new("use a::{used::Thing, inner};\n"));
const MODULE_BARE_USER: Source = Source::new(Located::new("crates/b/src/lib.rs"), Text::new("use a::used;\n"));

const CASES: [Case; 16] = [
    Case::new(Title::new("no dependent"), &[LONELY], Flagged::new(&[])),
    Case::new(Title::new("used by itself"), &[EXPORTER, USER, SELF_USER], Flagged::new(&["crates/a/src/lib.rs:2"])),
    Case::new(Title::new("named by its binary"), &[EXPORTER, USER, BINARY], Flagged::new(&[])),
    Case::new(Title::new("exporter alone"), &[EXPORTER], Flagged::new(&[])),
    Case::new(Title::new("named in a macro"), &[EXPORTER, MACRO_USER], Flagged::new(&[])),
    Case::new(Title::new("named by a glob"), &[EXPORTER, GLOB_USER], Flagged::new(&[])),
    Case::new(Title::new("a glob in a third crate"), &[EXPORTER, USER, OTHER_GLOB], Flagged::new(&[])),
    Case::new(
        Title::new("a glob of its own crate"),
        &[EXPORTER, USER, SELF_GLOB],
        Flagged::new(&["crates/a/src/lib.rs:2"]),
    ),
    Case::new(
        Title::new("named only by its own tests"),
        &[EXPORTER, USER, TEST_USER],
        Flagged::new(&["crates/a/src/lib.rs:2"]),
    ),
    Case::new(Title::new("named by tests and another crate"), &[EXPORTER, PATH_USER, TEST_USER_ALL], Flagged::new(&[])),
    Case::new(Title::new("crate-only import"), &[CRATE_ONLY, USER], Flagged::new(&[])),
    Case::new(
        Title::new("a public module no other crate names"),
        &[MODULES, MODULE_USER],
        Flagged::new(&["crates/a/src/lib.rs:1"]),
    ),
    Case::new(
        Title::new("a public module named by a path"),
        &[MODULES, MODULE_PATH_USER],
        Flagged::new(&["crates/a/src/lib.rs:1"]),
    ),
    Case::new(Title::new("public modules named in a group"), &[MODULES, MODULE_GROUP_USER], Flagged::new(&[])),
    Case::new(
        Title::new("a public module imported by name"),
        &[MODULES, MODULE_BARE_USER],
        Flagged::new(&["crates/a/src/lib.rs:1"]),
    ),
    Case::new(Title::new("a public module alone"), &[MODULES], Flagged::new(&[])),
];

#[test]
fn exports_no_other_crate_names_are_flagged() {
    assert_eq!(Sources::mismatches::<PublicSurface>(&CASES), Lines::default());
}
