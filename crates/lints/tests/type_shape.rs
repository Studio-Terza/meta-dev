mod common;

use common::Sources;
use lints::TypeShape;

const BOOL_FIELD: &str = "struct A { on: bool }\n";
const TUPLE: &str = "fn f(pair: (u8, u8)) {}\nfn g() -> (u8, u8) { (1, 2) }\n";
const STRING_ERROR: &str = "fn f() -> Result<u8, String> { Ok(1) }\n";
const FINE: &str = "struct A { count: u8 }\nfn f() -> Result<u8, std::fmt::Error> { Ok(1) }\nfn g() {}\n#[cfg(test)]\nmod tests {\n    struct B { on: bool }\n}\n";

#[test]
fn bool_fields_are_reported() {
    let report = TypeShape::report(&[Sources::file("a.rs", BOOL_FIELD)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}

#[test]
fn tuples_in_signatures_are_reported() {
    let report = TypeShape::report(&[Sources::file("a.rs", TUPLE)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:2"]);
}

#[test]
fn string_errors_are_reported() {
    let report = TypeShape::report(&[Sources::file("a.rs", STRING_ERROR)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}

#[test]
fn well_shaped_types_and_test_modules_pass() {
    assert!(TypeShape::report(&[Sources::file("a.rs", FINE)]).is_clean());
}
