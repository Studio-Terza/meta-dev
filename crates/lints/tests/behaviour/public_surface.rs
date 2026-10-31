use lints::PublicSurface;

use crate::common::{Case, Source, Sources};

const EXPORTER: Source = Source::new("crates/a/src/lib.rs", "mod inner;\npub use inner::{Used, Unused};\n");
const USER: Source = Source::new("crates/b/src/lib.rs", "use a::Used;\n");
const PATH_USER: Source = Source::new("crates/b/src/lib.rs", "fn f(x: a::Used, y: a::Unused) {}\n");
const SELF_USER: Source = Source::new("crates/a/src/inner.rs", "use a::Unused;\n");
const BINARY: Source = Source::new("crates/a/src/main.rs", "use a::Unused;\n");
const LONELY: Source = Source::new("crates/c/src/lib.rs", "mod inner;\npub use inner::Alone;\n");

const CASES: [Case; 6] = [
    Case { name: "one name used", sources: &[EXPORTER, USER], flagged: &["crates/a/src/lib.rs:2"] },
    Case { name: "both names used", sources: &[EXPORTER, PATH_USER], flagged: &[] },
    Case { name: "no dependent", sources: &[LONELY], flagged: &[] },
    Case { name: "used by itself", sources: &[EXPORTER, USER, SELF_USER], flagged: &["crates/a/src/lib.rs:2"] },
    Case { name: "named by its binary", sources: &[EXPORTER, USER, BINARY], flagged: &[] },
    Case { name: "exporter alone", sources: &[EXPORTER], flagged: &[] },
];

#[test]
fn exports_no_other_crate_names_are_flagged() {
    assert_eq!(Sources::mismatches::<PublicSurface>(&CASES), Vec::<String>::new());
}
