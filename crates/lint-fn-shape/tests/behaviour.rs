use lint_fn_shape::FnShape;

use lint_fixtures::{Case, Source, Sources};

const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";

const TEST_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n";
const GENERATED: &str = "struct A;\nmacro_rules! wide {\n    () => {\n        impl A {\n            fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n        }\n    };\n}\n";

const INLINE: &str = "mod inner {\n    pub fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const DEFAULT_METHOD: &str = "trait T {\n    fn f(a: u8, b: u8, c: u8, d: u8, e: u8) {}\n}\n";
const TRY_FOLD: &str = "fn f() {\n    let total = items.iter().try_fold((0, 0), |a, b| a);\n}\n";
const ARMS: &str = "fn f(a: u8) {\n    match a {\n        _ => match a {\n            _ => match a {\n                _ => g(),\n            },\n        },\n    }\n}\n";
const FORS: &str =
    "fn f(a: u8) {\n    for _ in a {\n        for _ in a {\n            for _ in a {}\n        }\n    }\n}\n";
const WHILES: &str = "fn f(a: u8) {\n    while a {\n        while a {\n            while a {}\n        }\n    }\n}\n";
const LOOPS: &str = "fn f(a: u8) {\n    loop {\n        loop {\n            loop {}\n        }\n    }\n}\n";
const CLOSURES: &str = "fn f() {\n    a(|| {\n        a(|| {\n            a(|| g());\n        });\n    });\n}\n";
const SIBLINGS: &str = "fn f(a: bool) {\n    if a {\n        if a {\n            g();\n        }\n    }\n    if a {\n        if a {\n            g();\n        }\n    }\n}\n";
const BOOLEANS: &str = "fn f(x: u8) {\n    let a = !b;\n    let c = true;\n    let d = (x > 1);\n    let e = x.is_empty();\n    let h = x.contains(1);\n    let i = x.len();\n}\n";

const CASES: [Case; 14] = [
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("test impl", &[Source::new("a.rs", TEST_IMPL)], &[]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:5"]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:2"]),
    Case::new("default method", &[Source::new("a.rs", DEFAULT_METHOD)], &["a.rs:2"]),
    Case::new("try_fold seed", &[Source::new("a.rs", TRY_FOLD)], &["a.rs:2"]),
    Case::new("match arms nest", &[Source::new("a.rs", ARMS)], &["a.rs:5"]),
    Case::new("for loops nest", &[Source::new("a.rs", FORS)], &["a.rs:4"]),
    Case::new("while loops nest", &[Source::new("a.rs", WHILES)], &["a.rs:4"]),
    Case::new("loops nest", &[Source::new("a.rs", LOOPS)], &["a.rs:4"]),
    Case::new("closures nest", &[Source::new("a.rs", CLOSURES)], &["a.rs:4"]),
    Case::new("sibling blocks do not add up", &[Source::new("a.rs", SIBLINGS)], &[]),
    Case::new(
        "each boolean form",
        &[Source::new("a.rs", BOOLEANS)],
        &["a.rs:2", "a.rs:3", "a.rs:4", "a.rs:5", "a.rs:6"],
    ),
];

#[test]
fn hairy_control_flow_is_flagged() {
    assert_eq!(Sources::mismatches::<FnShape>(&CASES), Vec::<String>::new());
}
