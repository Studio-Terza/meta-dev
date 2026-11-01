use lint_const_shape::ConstShape;
use lint_core::Lint;
use lint_fixtures::{Case, Source, Sources};

const NOT_TEST: &str = "#[cfg(not(test))]\nmod real {\n    pub fn f(x: u8) -> u32 { x as u32 }\n}\n";
const NESTED: &str = "pub struct A;\nconst _: () = {\n    impl A {\n        pub fn wide(x: u8) -> u32 { x as u32 }\n    }\n};\nconst fn outer() {\n    fn inner() { for _ in 0..3 {} }\n}\n";

#[test]
fn modules_compiled_outside_tests_are_checked() {
    let report = ConstShape::report(&[Sources::file("a.rs", NOT_TEST)]);
    assert_eq!(Sources::lines(&report), ["a.rs:3"]);
}

#[test]
fn const_context_does_not_leak_into_nested_functions() {
    let report = ConstShape::report(&[Sources::file("a.rs", NESTED)]);
    assert_eq!(Sources::lines(&report), ["a.rs:4"]);
}

const BITS_CAST: &str = "fn f(x: u8) -> u32 { x as u32 }\n";
const INLINE_BITS: &str = "mod bits {\n    pub fn f(x: u8) -> u32 { x as u32 }\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f(x: u8) -> u32 { x as u32 }\n}\n";
const BITS_LOOP: &str = "mod bits {\n    pub const fn f() {\n        for _ in 0..3 {}\n    }\n}\n";
const BITS_LEAK: &str = "mod bits {}\nfn f(x: u8) -> u32 { x as u32 }\n";

const PLATFORM_MODULE: &str = "#[cfg(unix)]\nmod real {\n    pub fn f(x: u8) -> u32 { x as u32 }\n}\n";
const TRAIT_FN: &str = "pub trait T {\n    fn f(x: u8) -> u32 { x as u32 }\n}\n";
const IMPL_CONST: &str = "struct A;\nimpl A {\n    const X: u32 = { let mut i = 0; while i < 3 { i += 1; } i };\n}\n";
const TRAIT_CONST: &str = "pub trait T {\n    const X: u32 = { let mut i = 0; while i < 3 { i += 1; } i };\n}\n";
const CONST_BLOCK: &str = "fn f() -> u32 {\n    const { let mut i = 0; while i < 3 { i += 1; } i }\n}\n";
const ARRAY_LENGTH: &str = "pub struct A {\n    pub b: [u8; { let mut i = 0; while i < 3 { i += 1; } i }],\n}\n";
const REPEAT_LENGTH: &str = "fn f() -> [u8; 3] {\n    [0; { let mut i = 0; while i < 3 { i += 1; } i }]\n}\n";
const DISCRIMINANT: &str = "enum E {\n    A = { let mut i = 0; while i < 3 { i += 1; } i },\n}\n";

const CASES: [Case; 15] = [
    Case::new("bits module file", &[Source::new("crates/x/src/bits/mask.rs", BITS_CAST)], &[]),
    Case::new("bits module root", &[Source::new("crates/x/src/bits.rs", BITS_CAST)], &[]),
    Case::new("inline bits module", &[Source::new("a.rs", INLINE_BITS)], &[]),
    Case::new("inline other module", &[Source::new("a.rs", INLINE_OTHER)], &["a.rs:2"]),
    Case::new(
        "other module file",
        &[Source::new("crates/x/src/masks/mask.rs", BITS_CAST)],
        &["crates/x/src/masks/mask.rs:1"],
    ),
    Case::new("bits keep the loop rule", &[Source::new("a.rs", BITS_LOOP)], &["a.rs:3"]),
    Case::new("bits do not leak out", &[Source::new("a.rs", BITS_LEAK)], &["a.rs:2"]),
    Case::new("a platform gate is not a test gate", &[Source::new("a.rs", PLATFORM_MODULE)], &["a.rs:3"]),
    Case::new("a default trait fn", &[Source::new("a.rs", TRAIT_FN)], &["a.rs:2"]),
    Case::new("an impl const", &[Source::new("a.rs", IMPL_CONST)], &["a.rs:3"]),
    Case::new("a trait const", &[Source::new("a.rs", TRAIT_CONST)], &["a.rs:2"]),
    Case::new("a const block", &[Source::new("a.rs", CONST_BLOCK)], &["a.rs:2"]),
    Case::new("an array length", &[Source::new("a.rs", ARRAY_LENGTH)], &["a.rs:2"]),
    Case::new("a repeat length", &[Source::new("a.rs", REPEAT_LENGTH)], &["a.rs:2"]),
    Case::new("a discriminant", &[Source::new("a.rs", DISCRIMINANT)], &["a.rs:2"]),
];

#[test]
fn casts_are_allowed_only_inside_a_bits_module() {
    assert_eq!(Sources::mismatches::<ConstShape>(&CASES), Vec::<String>::new());
}
