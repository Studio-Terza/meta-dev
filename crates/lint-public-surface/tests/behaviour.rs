use lint_public_surface::PublicSurface;

use lint_fixtures::{Case, Source, Sources};

const EXPORTER: Source = Source::new("crates/a/src/lib.rs", "mod inner;\npub use inner::{Used, Unused};\n");
const USER: Source = Source::new("crates/b/src/lib.rs", "use a::Used;\n");
const PATH_USER: Source = Source::new("crates/b/src/lib.rs", "fn f(x: a::Used, y: a::Unused) {}\n");
const SELF_USER: Source = Source::new("crates/a/src/inner.rs", "use a::Unused;\n");
const BINARY: Source = Source::new("crates/a/src/main.rs", "use a::Unused;\n");
const LONELY: Source = Source::new("crates/c/src/lib.rs", "mod inner;\npub use inner::Alone;\n");

const MACRO_USER: Source =
    Source::new("crates/b/src/lib.rs", "use a::Used;\nfn f() {\n    println!(\"{:?}\", a::Unused);\n}\n");
const GLOB_USER: Source = Source::new("crates/b/src/lib.rs", "use a::*;\n");
const CRATE_ONLY: Source =
    Source::new("crates/a/src/lib.rs", "mod inner;\npub use inner::Used;\npub(crate) use inner::Hidden;\n");

const TEST_USER: Source = Source::new("crates/a/tests/api.rs", "use a::Unused;\n");
const TEST_USER_ALL: Source = Source::new("crates/a/tests/api.rs", "use a::{Used, Unused};\n");

const CASES: [Case; 11] = [
    Case::new("one name used", &[EXPORTER, USER], &["crates/a/src/lib.rs:2"]),
    Case::new("both names used", &[EXPORTER, PATH_USER], &[]),
    Case::new("no dependent", &[LONELY], &[]),
    Case::new("used by itself", &[EXPORTER, USER, SELF_USER], &["crates/a/src/lib.rs:2"]),
    Case::new("named by its binary", &[EXPORTER, USER, BINARY], &[]),
    Case::new("exporter alone", &[EXPORTER], &[]),
    Case::new("named in a macro", &[EXPORTER, MACRO_USER], &[]),
    Case::new("named by a glob", &[EXPORTER, GLOB_USER], &[]),
    Case::new("named only by its own tests", &[EXPORTER, USER, TEST_USER], &["crates/a/src/lib.rs:2"]),
    Case::new("named by tests and another crate", &[EXPORTER, PATH_USER, TEST_USER_ALL], &[]),
    Case::new("crate-only import", &[CRATE_ONLY, USER], &[]),
];

#[test]
fn exports_no_other_crate_names_are_flagged() {
    assert_eq!(Sources::mismatches::<PublicSurface>(&CASES), Vec::<String>::new());
}
