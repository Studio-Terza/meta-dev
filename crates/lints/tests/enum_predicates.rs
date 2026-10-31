mod common;

use common::Sources;
use lints::EnumPredicates;

const ENUM: &str =
    "enum Mode { A, B }\nimpl Mode {\n    fn is_a(&self) -> bool { true }\n    fn name(&self) -> u8 { 0 }\n}\n";
const STRUCT: &str = "struct Mode;\nimpl Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const IMPL_ONLY: &str = "impl Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const DECLARES: &str = "pub enum Mode { A }\n";

#[test]
fn bool_methods_on_enums_are_reported() {
    let report = EnumPredicates::report(&[Sources::file("a.rs", ENUM)]);
    assert_eq!(Sources::lines(&report), ["a.rs:3"]);
}

#[test]
fn bool_methods_on_structs_pass() {
    assert!(EnumPredicates::report(&[Sources::file("a.rs", STRUCT)]).is_clean());
}

#[test]
fn an_enum_declared_in_another_file_of_the_same_crate_counts() {
    let files = [Sources::file("x/src/mode.rs", DECLARES), Sources::file("x/src/impls.rs", IMPL_ONLY)];
    assert_eq!(Sources::lines(&EnumPredicates::report(&files)), ["x/src/impls.rs:2"]);
}

#[test]
fn an_enum_in_another_crate_does_not_count() {
    let files = [Sources::file("x/src/mode.rs", DECLARES), Sources::file("y/src/impls.rs", IMPL_ONLY)];
    assert!(EnumPredicates::report(&files).is_clean());
}
