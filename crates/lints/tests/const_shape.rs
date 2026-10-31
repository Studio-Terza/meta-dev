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
