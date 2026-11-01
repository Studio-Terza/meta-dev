use lint_fn_shape::FnShape;

use lint_fixtures::{Case, Source, Sources};

const DEEP: &str = "fn f(a: bool, b: bool, c: bool) {\n    if a {\n        if b {\n            if c {\n                g();\n            }\n        }\n    }\n}\n";
const SHALLOW: &str = "fn f(a: bool, b: bool) {\n    if a {\n        if b {\n            g();\n        }\n    }\n}\n";
const WIDE: &str = "fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n";
const NARROW: &str = "fn f(a: u8, b: u8, c: u8, d: u8) {}\n";
const BOOLEAN: &str = "fn f(x: u8) {\n    let ready = x > 1;\n    let typed: bool = g();\n}\n";
const TUPLE: &str = "fn f() {\n    let pair = (1, 2);\n    let total = items.iter().fold((0, 0), |a, b| a);\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";

const TEST_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n";
const GENERATED: &str = "struct A;\nmacro_rules! wide {\n    () => {\n        impl A {\n            fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n        }\n    };\n}\n";

const CASES: [Case; 10] = [
    Case::new("deep", &[Source::new("a.rs", DEEP)], &["a.rs:4"]),
    Case::new("shallow", &[Source::new("a.rs", SHALLOW)], &[]),
    Case::new("wide", &[Source::new("a.rs", WIDE)], &["a.rs:1"]),
    Case::new("narrow", &[Source::new("a.rs", NARROW)], &[]),
    Case::new("boolean locals", &[Source::new("a.rs", BOOLEAN)], &["a.rs:2", "a.rs:3"]),
    Case::new("tuples", &[Source::new("a.rs", TUPLE)], &["a.rs:2", "a.rs:3"]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("test impl", &[Source::new("a.rs", TEST_IMPL)], &[]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:5"]),
];

#[test]
fn hairy_control_flow_is_flagged() {
    assert_eq!(Sources::mismatches::<FnShape>(&CASES), Vec::<String>::new());
}
