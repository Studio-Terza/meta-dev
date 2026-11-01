use lint_core::Lint;
use lint_enum_predicates::EnumPredicates;
use lint_fixtures::{Case, Source, Sources};

const IMPL_ONLY: &str = "impl Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const DECLARES: &str = "pub enum Mode { A }\n";

#[test]
fn an_enum_in_another_crate_does_not_count() {
    let files = [Sources::file("x/src/mode.rs", DECLARES), Sources::file("y/src/impls.rs", IMPL_ONLY)];
    assert!(EnumPredicates::report(&files).is_clean());
}

const SHARED_TEST_NAME: &str = "enum Mode { A }\n";
const OTHER_TEST_CRATE: &str = "struct Mode;\nimpl Mode {\n    fn ok(&self) -> bool { true }\n}\n";
const STATUS_ENUM: &str = "pub enum Status { On }\n";
const STATUS_STRUCT: &str = "pub struct Status;\nimpl Status {\n    pub fn ok(&self) -> bool { true }\n}\n";

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

const SAME_NAME_NEARBY: &str = "mod b {\n    pub struct Mode;\n}\nmod a {\n    pub enum Mode { A }\n    impl Mode {\n        pub fn is_a(&self) -> bool { true }\n    }\n}\n";
const PATH_TO_A_STRUCT: &str =
    "mod a {\n    pub struct Mode;\n}\nenum Mode { A }\nimpl a::Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const A_UNION_ALSO_NAMED: &str = "mod a {\n    pub enum Mode { A }\n}\nmod b {\n    pub union Mode { x: u8 }\n}\nimpl a::Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const ENUM_IN_MACRO: &str = "wrap! {\n    enum Mode { A }\n}\nimpl Mode {\n    fn is_a(&self) -> bool { true }\n}\n";
const IMPL_IN_MACRO: &str =
    "enum Mode { A }\nwrap! {\n    impl Mode {\n        fn is_a(&self) -> bool { true }\n    }\n}\n";

const CASES: [Case; 5] = [
    Case::new("same name in the next module", &[Source::new("a.rs", SAME_NAME_NEARBY)], &["a.rs:7"]),
    Case::new("a path to a struct", &[Source::new("a.rs", PATH_TO_A_STRUCT)], &[]),
    Case::new("a union of that name", &[Source::new("a.rs", A_UNION_ALSO_NAMED)], &[]),
    Case::new("enum inside a macro", &[Source::new("a.rs", ENUM_IN_MACRO)], &["a.rs:5"]),
    Case::new("impl inside a macro", &[Source::new("a.rs", IMPL_IN_MACRO)], &["a.rs:4"]),
];

#[test]
fn a_name_resolves_to_the_nearest_declaration() {
    assert_eq!(Sources::mismatches::<EnumPredicates>(&CASES), Vec::<String>::new());
}
