use crate::common::Sources;
use lints::{Lint, TypeShape};

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

const VARIANTS: &str = "pub enum Cmd {\n    Set { on: bool },\n    Pair((u8, u8)),\n    Bad(Result<u8, String>),\n}\n";
const SPELLINGS: &str = "pub fn f() -> Result<u8, std::string::String> { Ok(1) }\npub struct A { on: std::primitive::bool }\npub type R = Result<u8, String>;\npub type P = (u8, u8);\n";
const MACRO: &str = "macro_rules! m { () => { fn f() -> (u8, u8) { (1, 2) } } }\n";
const NOT_TEST: &str = "#[cfg(not(test))]\nmod real {\n    pub struct S { pub on: bool }\n}\n";
const ATTRIBUTED: &str = "struct A {\n    #[allow(unused)]\n    #[allow(dead_code)]\n    pub on: bool,\n}\n";

#[test]
fn enum_variant_fields_are_checked() {
    let report = TypeShape::report(&[Sources::file("a.rs", VARIANTS)]);
    assert_eq!(Sources::lines(&report), ["a.rs:2", "a.rs:3", "a.rs:4"]);
}

#[test]
fn equivalent_spellings_and_aliases_are_checked() {
    let report = TypeShape::report(&[Sources::file("a.rs", SPELLINGS)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1", "a.rs:2", "a.rs:3", "a.rs:4"]);
}

#[test]
fn macro_rules_bodies_are_checked() {
    let report = TypeShape::report(&[Sources::file("a.rs", MACRO)]);
    assert_eq!(Sources::lines(&report), ["a.rs:1"]);
}

#[test]
fn modules_compiled_outside_tests_are_checked() {
    let report = TypeShape::report(&[Sources::file("a.rs", NOT_TEST)]);
    assert_eq!(Sources::lines(&report), ["a.rs:3"]);
}

#[test]
fn a_field_is_reported_on_its_own_line_not_its_attribute_line() {
    let report = TypeShape::report(&[Sources::file("a.rs", ATTRIBUTED)]);
    assert_eq!(Sources::lines(&report), ["a.rs:4"]);
}
