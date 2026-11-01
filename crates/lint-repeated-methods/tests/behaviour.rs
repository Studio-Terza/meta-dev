use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Source, Sources};

const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const DERIVED: &str = "#[derive(Newtype)]\npub struct A(u8);\n#[derive(Newtype)]\npub struct B(u8);\n#[derive(Newtype)]\npub struct C(u8);\n";
const CONSTRUCTORS: &str = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u8) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u8) -> Self { C(inner) }\n}\n";

const OWN_TYPE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn join(&self, other: A) -> A { other }\n}\nimpl B {\n    pub fn join(&self, other: B) -> B { other }\n}\nimpl C {\n    pub fn join(&self, other: C) -> C { other }\n}\n";
const DIFFERENT_RETURNS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u16 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u32 { 1 }\n}\n";
const INLINE: &str = "mod inner {\n    struct A;\n    struct B;\n    struct C;\n    impl A {\n        pub fn show(&self) -> u8 { 1 }\n    }\n    impl B {\n        pub fn show(&self) -> u8 { 1 }\n    }\n    impl C {\n        pub fn show(&self) -> u8 { 1 }\n    }\n}\n";

const CASES: [Case; 7] = [
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPLS)], &[]),
    Case::new("derived newtypes", &[Source::new("a.rs", DERIVED)], &[]),
    Case::new("hand-written constructors", &[Source::new("a.rs", CONSTRUCTORS)], &["a.rs:5"]),
    Case::new("own type erased", &[Source::new("a.rs", OWN_TYPE)], &["a.rs:5"]),
    Case::new("different returns", &[Source::new("a.rs", DIFFERENT_RETURNS)], &[]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:6"]),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}
