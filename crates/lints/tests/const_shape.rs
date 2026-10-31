mod common;

use common::Sources;
use lints::ConstShape;

const CAST: &str = "fn f(x: u8) -> u32 { x as u32 }\nconst fn g(x: u8) -> u32 { x as u32 }\n";
const LOOPS: &str =
    "const fn f() {\n    let mut i = 0;\n    while i < 3 { i += 1; }\n    loop { break; }\n    for _ in 0..3 {}\n}\n";
const WALK: &str = "const fn f(items: &[u8]) {\n    let mut rest = items;\n    while let [_, tail @ ..] = rest { rest = tail; }\n}\nfn g() { for _ in 0..3 {} }\n";

#[test]
fn casts_outside_const_fns_are_reported() {
    let report = ConstShape::report(&[Sources::file("a.rs", CAST)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}

#[test]
fn counter_open_and_for_loops_in_const_context_are_reported() {
    let report = ConstShape::report(&[Sources::file("a.rs", LOOPS)]);
    assert_eq!(Sources::lines(&report), ["a.rs:3", "a.rs:4", "a.rs:5"]);
}

#[test]
fn slice_walks_in_const_fns_and_loops_outside_them_pass() {
    assert!(ConstShape::report(&[Sources::file("a.rs", WALK)]).is_clean());
}

const MACRO_CAST: &str = "fn show(x: u8) -> String {\n    format!(\"{}\", x as u32)\n}\n";
const NOT_TEST: &str = "#[cfg(not(test))]\nmod real {\n    pub fn f(x: u8) -> u32 { x as u32 }\n}\n";
const NESTED: &str = "pub struct A;\nconst _: () = {\n    impl A {\n        pub fn wide(x: u8) -> u32 { x as u32 }\n    }\n};\nconst fn outer() {\n    fn inner() { for _ in 0..3 {} }\n}\n";
const ITEMS: &str = "pub static X: u32 = 1u8 as u32;\npub static Y: u32 = { let mut i = 0; while i < 3 { i += 1; } i };\npub trait T { const X: u32 = 1u8 as u32; }\npub struct A { pub b: [u8; N as usize] }\nenum E { A = 1u8 as isize }\n";

#[test]
fn casts_inside_macro_arguments_are_reported() {
    let report = ConstShape::report(&[Sources::file("a.rs", MACRO_CAST)]);
    assert_eq!(Sources::lines(&report), ["a.rs:2"]);
}

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

#[test]
fn statics_trait_consts_lengths_and_discriminants_are_const_context() {
    let report = ConstShape::report(&[Sources::file("a.rs", ITEMS)]);
    assert_eq!(Sources::lines(&report), ["a.rs:2"]);
}
