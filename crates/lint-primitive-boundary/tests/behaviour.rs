use lint_primitive_boundary::PrimitiveBoundary;

use lint_fixtures::{Case, Source, Sources};

const BARE: &str = "fn f(x: u32) -> f64 {\n    g(x)\n}\n";
const NEWTYPE: &str = "struct Count(u32);\nfn f(x: u32) -> u8 {\n    g(x)\n}\n";
const SIZES: &str = "fn f(items: usize, len: usize) {}\nstruct A {\n    length: usize,\n    total: usize,\n}\n";
const WIRE: &str = "use serde_json::Value;\nfn f(x: Value) {}\nfn g(x: serde_json::Value) {}\n";
const TEXT: &str = "fn f(x: &str) -> String {\n    let t = x.trim();\n    t.to_string()\n}\n";
const DOOR: &str = "struct A;\nimpl std::str::FromStr for A {\n    type Err = ();\n    fn from_str(x: &str) -> Result<A, ()> {\n        Ok(A).map(|a| x.trim())\n    }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(x: u32) {}\n}\n";

const INLINE_BITS: &str = "mod bits {\n    pub fn f(x: u32) {}\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f(x: u32) {}\n}\n";

const CASES: [Case; 12] = [
    Case::new("bare primitives", &[Source::new("a.rs", BARE)], &["a.rs:1", "a.rs:1"]),
    Case::new("newtype file", &[Source::new("a.rs", NEWTYPE)], &[]),
    Case::new("sizes", &[Source::new("a.rs", SIZES)], &["a.rs:1", "a.rs:1", "a.rs:4"]),
    Case::new("wire types", &[Source::new("a.rs", WIRE)], &["a.rs:2", "a.rs:3"]),
    Case::new("text outside a door", &[Source::new("a.rs", TEXT)], &["a.rs:2", "a.rs:3"]),
    Case::new("text in a door", &[Source::new("a.rs", DOOR)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("bits module file", &[Source::new("crates/x/src/bits/mask.rs", BARE)], &[]),
    Case::new("bits module root", &[Source::new("crates/x/src/bits.rs", BARE)], &[]),
    Case::new("inline bits module", &[Source::new("a.rs", INLINE_BITS)], &[]),
    Case::new("inline other module", &[Source::new("a.rs", INLINE_OTHER)], &["a.rs:2"]),
    Case::new(
        "other module file",
        &[Source::new("crates/x/src/masks/mask.rs", BARE)],
        &["crates/x/src/masks/mask.rs:1", "crates/x/src/masks/mask.rs:1"],
    ),
];

#[test]
fn primitives_away_from_a_types_boundary_are_flagged() {
    assert_eq!(Sources::mismatches::<PrimitiveBoundary>(&CASES), Vec::<String>::new());
}
