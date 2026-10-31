use lints::FnShape;

use crate::common::{Case, Source, Sources};

const DEEP: &str = "fn f(a: bool, b: bool, c: bool) {\n    if a {\n        if b {\n            if c {\n                g();\n            }\n        }\n    }\n}\n";
const SHALLOW: &str = "fn f(a: bool, b: bool) {\n    if a {\n        if b {\n            g();\n        }\n    }\n}\n";
const WIDE: &str = "fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n";
const NARROW: &str = "fn f(a: u8, b: u8, c: u8, d: u8) {}\n";
const BOOLEAN: &str = "fn f(x: u8) {\n    let ready = x > 1;\n    let typed: bool = g();\n}\n";
const TUPLE: &str = "fn f() {\n    let pair = (1, 2);\n    let total = items.iter().fold((0, 0), |a, b| a);\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";

const CASES: [Case; 7] = [
    Case { name: "deep", sources: &[Source::new("a.rs", DEEP)], flagged: &["a.rs:4"] },
    Case { name: "shallow", sources: &[Source::new("a.rs", SHALLOW)], flagged: &[] },
    Case { name: "wide", sources: &[Source::new("a.rs", WIDE)], flagged: &["a.rs:1"] },
    Case { name: "narrow", sources: &[Source::new("a.rs", NARROW)], flagged: &[] },
    Case { name: "boolean locals", sources: &[Source::new("a.rs", BOOLEAN)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "tuples", sources: &[Source::new("a.rs", TUPLE)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
];

#[test]
fn hairy_control_flow_is_flagged() {
    assert_eq!(Sources::mismatches::<FnShape>(&CASES), Vec::<String>::new());
}
