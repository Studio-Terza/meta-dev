use lint_public_surface::PublicSurface;

use lint_fixtures::{Case, Source, Sources};

const EXPORTER: Source = Source::new("crates/a/src/lib.rs", "mod inner;\npub use inner::{Used, Unused};\n");
const USER: Source = Source::new("crates/b/src/lib.rs", "use a::Used;\n");
const SELF_USER: Source = Source::new("crates/a/src/inner.rs", "use a::Unused;\n");
const BINARY: Source = Source::new("crates/a/src/main.rs", "use a::Unused;\n");
const LONELY: Source = Source::new("crates/c/src/lib.rs", "mod inner;\npub use inner::Alone;\n");

const MACRO_USER: Source =
    Source::new("crates/b/src/lib.rs", "use a::Used;\nfn f() {\n    println!(\"{:?}\", a::Unused);\n}\n");
const GLOB_USER: Source = Source::new("crates/b/src/lib.rs", "use a::*;\n");
const CRATE_ONLY: Source =
    Source::new("crates/a/src/lib.rs", "mod inner;\npub use inner::Used;\npub(crate) use inner::Hidden;\n");

const OTHER_GLOB: Source = Source::new("crates/c/src/lib.rs", "use a::*;\n");
const SELF_GLOB: Source = Source::new("crates/a/src/inner.rs", "use a::*;\n");

const CASES: [Case; 9] = [
    Case::new("no dependent", &[LONELY], &[]),
    Case::new("used by itself", &[EXPORTER, USER, SELF_USER], &["crates/a/src/lib.rs:2"]),
    Case::new("named by its binary", &[EXPORTER, USER, BINARY], &[]),
    Case::new("exporter alone", &[EXPORTER], &[]),
    Case::new("named in a macro", &[EXPORTER, MACRO_USER], &[]),
    Case::new("named by a glob", &[EXPORTER, GLOB_USER], &[]),
    Case::new("a glob in a third crate", &[EXPORTER, USER, OTHER_GLOB], &[]),
    Case::new("a glob of its own crate", &[EXPORTER, USER, SELF_GLOB], &["crates/a/src/lib.rs:2"]),
    Case::new("crate-only import", &[CRATE_ONLY, USER], &[]),
];

#[test]
fn exports_no_other_crate_names_are_flagged() {
    assert_eq!(Sources::mismatches::<PublicSurface>(&CASES), Vec::<String>::new());
}
