use lint_expression_flow::ExpressionFlow;

use lint_fixtures::{Case, Source, Sources};

const CONST_LOOP: &str = "const fn f() -> u8 {\n    let mut i = 0;\n    while i < 3 { i += 1; }\n    i\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() {\n        for _ in [1] {}\n    }\n}\n";

const WRITTEN: &str =
    "fn f() -> String {\n    let mut out = String::new();\n    write!(out, \"x\").ok();\n    out\n}\n";
const CLOSURE: &str =
    "fn f(limit: u8) {\n    let mut seen = Seen::default();\n    let mut bump = || seen.note(limit);\n    bump();\n}\n";
const TEST_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    fn f() {\n        for _ in [1] {}\n    }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! looping {\n    () => {\n        impl A {\n            fn f() {\n                for _ in [1] {}\n            }\n        }\n    };\n}\n";

const INLINE: &str = "mod inner {\n    pub fn f() {\n        loop {}\n    }\n}\n";
const DEFAULT_METHOD: &str = "trait T {\n    fn f(&self) {\n        loop {}\n    }\n}\n";
const IMMUTABLE: &str = "fn f() -> u8 {\n    let total = 1;\n    total\n}\n";
const LENT_MUTABLY: &str = "fn f() -> u8 {\n    let mut total = 1;\n    g(&mut total);\n    total\n}\n";
const LENT_SHARED: &str = "fn f() -> u8 {\n    let mut total = 1;\n    g(&total);\n    total\n}\n";
const PLACES: &str = "fn f(mut a: A) {\n    a.b = 1;\n    a[0] = 2;\n    (a) = 3;\n}\n";

const CASES: [Case; 12] = [
    Case::new("const loop", &[Source::new("a.rs", CONST_LOOP)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("written by a macro", &[Source::new("a.rs", WRITTEN)], &[]),
    Case::new("called closure", &[Source::new("a.rs", CLOSURE)], &[]),
    Case::new("test impl", &[Source::new("a.rs", TEST_IMPL)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:6"]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:3"]),
    Case::new("default method", &[Source::new("a.rs", DEFAULT_METHOD)], &["a.rs:3"]),
    Case::new("immutable local", &[Source::new("a.rs", IMMUTABLE)], &[]),
    Case::new("lent mutably", &[Source::new("a.rs", LENT_MUTABLY)], &[]),
    Case::new("lent shared", &[Source::new("a.rs", LENT_SHARED)], &["a.rs:2"]),
    Case::new("fields, elements and parentheses", &[Source::new("a.rs", PLACES)], &["a.rs:2", "a.rs:3", "a.rs:4"]),
];

#[test]
fn control_flow_that_is_not_an_expression_is_flagged() {
    assert_eq!(Sources::mismatches::<ExpressionFlow>(&CASES), Vec::<String>::new());
}
