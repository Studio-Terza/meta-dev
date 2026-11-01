use lint_bool_methods::BoolMethods;
use lint_core::Lint;
use lint_fixtures::{Case, Source, Sources};

const LISTED: &str = "pub struct A;\nimpl A {\n    pub fn is_on(&self) -> bool { true }\n    pub fn has_room(&self) -> bool { true }\n}\n";
const NESTED: &str =
    "mod inner {\n    pub struct A;\n    impl A {\n        pub fn is_on(&self) -> bool { true }\n    }\n}\n";
const PRIVATE: &str = "pub struct A;\nimpl A {\n    fn is_on(&self) -> bool { true }\n    pub(crate) fn has_room(&self) -> bool { true }\n}\n";
const OTHER_NAMES: &str = "pub struct A;\nimpl A {\n    pub fn allows(&self) -> bool { true }\n    pub fn island(&self) -> bool { true }\n}\n";
const OTHER_RETURNS: &str =
    "pub struct A;\nimpl A {\n    pub fn is_on(&self) -> Option<u8> { None }\n    pub fn has_room(&self) {}\n}\n";
const TRAIT_IMPL: &str = "pub struct A;\nimpl Tr for A {\n    fn is_on(&self) -> bool { true }\n}\n";
const TEST_MODULE: &str = "pub struct A;\n#[cfg(test)]\nmod tests {\n    use super::A;\n    impl A {\n        pub fn is_on(&self) -> bool { true }\n    }\n}\n";
const TEST_IMPL: &str = "pub struct A;\n#[cfg(test)]\nimpl A {\n    pub fn is_on(&self) -> bool { true }\n}\n";

const PUBLIC_TRAIT: &str =
    "pub trait T {\n    fn is_on(&self) -> bool;\n    fn has_room(&self) -> bool;\n    fn size(&self) -> u8;\n}\n";
const PRIVATE_TRAIT: &str = "trait T {\n    fn is_on(&self) -> bool;\n}\n";
const TEST_TRAIT: &str = "#[cfg(test)]\npub trait T {\n    fn is_on(&self) -> bool;\n}\n";

const CASES: [Case; 12] = [
    Case::new("is_ and has_", &[Source::new("a.rs", LISTED)], &["a.rs:3", "a.rs:4"]),
    Case::new("inline module", &[Source::new("a.rs", NESTED)], &["a.rs:4"]),
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("other names", &[Source::new("a.rs", OTHER_NAMES)], &[]),
    Case::new("other returns", &[Source::new("a.rs", OTHER_RETURNS)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPL)], &[]),
    Case::new("test modules", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("test impls", &[Source::new("a.rs", TEST_IMPL)], &[]),
    Case::new("test targets", &[Source::new("x/tests/a.rs", LISTED)], &[]),
    Case::new("public trait", &[Source::new("a.rs", PUBLIC_TRAIT)], &["a.rs:2", "a.rs:3"]),
    Case::new("private trait", &[Source::new("a.rs", PRIVATE_TRAIT)], &[]),
    Case::new("test trait", &[Source::new("a.rs", TEST_TRAIT)], &[]),
];

#[test]
fn public_boolean_predicates_are_listed() {
    assert_eq!(Sources::mismatches::<BoolMethods>(&CASES), Vec::<String>::new());
}

#[test]
fn the_message_names_the_replacements() {
    let shown = BoolMethods::report(&[Sources::file("a.rs", LISTED)]).to_string();
    assert!(["Option", "enum", "NonEmpty", "fn is_on -> bool"].iter().all(|word| shown.contains(word)));
}
