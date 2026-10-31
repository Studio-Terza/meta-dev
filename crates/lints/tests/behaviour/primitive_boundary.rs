use lints::PrimitiveBoundary;

use crate::common::{Case, Source, Sources};

const BARE: &str = "fn f(x: u32) -> f64 {\n    g(x)\n}\n";
const NEWTYPE: &str = "struct Count(u32);\nfn f(x: u32) -> u8 {\n    g(x)\n}\n";
const SIZES: &str = "fn f(items: usize, len: usize) {}\nstruct A {\n    length: usize,\n    total: usize,\n}\n";
const WIRE: &str = "use serde_json::Value;\nfn f(x: Value) {}\nfn g(x: serde_json::Value) {}\n";
const TEXT: &str = "fn f(x: &str) -> String {\n    let t = x.trim();\n    t.to_string()\n}\n";
const DOOR: &str = "struct A;\nimpl std::str::FromStr for A {\n    type Err = ();\n    fn from_str(x: &str) -> Result<A, ()> {\n        Ok(A).map(|a| x.trim())\n    }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(x: u32) {}\n}\n";

const CASES: [Case; 7] = [
    Case { name: "bare primitives", sources: &[Source::new("a.rs", BARE)], flagged: &["a.rs:1", "a.rs:1"] },
    Case { name: "newtype file", sources: &[Source::new("a.rs", NEWTYPE)], flagged: &[] },
    Case { name: "sizes", sources: &[Source::new("a.rs", SIZES)], flagged: &["a.rs:1", "a.rs:1", "a.rs:4"] },
    Case { name: "wire types", sources: &[Source::new("a.rs", WIRE)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "text outside a door", sources: &[Source::new("a.rs", TEXT)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "text in a door", sources: &[Source::new("a.rs", DOOR)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
];

#[test]
fn primitives_away_from_a_types_boundary_are_flagged() {
    assert_eq!(Sources::mismatches::<PrimitiveBoundary>(&CASES), Vec::<String>::new());
}
