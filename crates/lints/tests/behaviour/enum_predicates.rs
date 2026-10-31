use crate::common::Sources;
use lints::{EnumPredicates, Lint};

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

const INLINE: &str =
    "mod inner {\n    pub enum Mode { A }\n    impl Mode {\n        pub fn is_a(&self) -> bool { true }\n    }\n}\n";
const SHARED_TEST_NAME: &str = "enum Mode { A }\n";
const OTHER_TEST_CRATE: &str = "struct Mode;\nimpl Mode {\n    fn ok(&self) -> bool { true }\n}\n";
const STATUS_ENUM: &str = "pub enum Status { On }\n";
const STATUS_STRUCT: &str = "pub struct Status;\nimpl Status {\n    pub fn ok(&self) -> bool { true }\n}\n";

#[test]
fn impls_inside_inline_modules_are_checked() {
    let report = EnumPredicates::report(&[Sources::file("a.rs", INLINE)]);
    assert_eq!(Sources::lines(&report), ["a.rs:4"]);
}

#[test]
fn integration_tests_are_separate_crates() {
    let files = [Sources::file("x/tests/a.rs", SHARED_TEST_NAME), Sources::file("y/tests/b.rs", OTHER_TEST_CRATE)];
    assert!(EnumPredicates::report(&files).is_clean());
}

#[test]
fn a_struct_sharing_an_enums_name_in_another_module_passes() {
    let files = [Sources::file("x/src/a.rs", STATUS_ENUM), Sources::file("x/src/b.rs", STATUS_STRUCT)];
    assert!(EnumPredicates::report(&files).is_clean());
}
