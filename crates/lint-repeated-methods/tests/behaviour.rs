use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Source, Sources};

const THREE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const TWO: &str = "struct A;\nstruct B;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const DERIVED: &str = "#[derive(Newtype)]\npub struct A(u8);\n#[derive(Newtype)]\npub struct B(u8);\n#[derive(Newtype)]\npub struct C(u8);\n";
const CONSTRUCTORS: &str = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u8) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u8) -> Self { C(inner) }\n}\n";

const CASES: [Case; 6] = [
    Case::new("three types", &[Source::new("a.rs", THREE)], &["a.rs:5"]),
    Case::new("two types", &[Source::new("a.rs", TWO)], &[]),
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPLS)], &[]),
    Case::new("derived newtypes", &[Source::new("a.rs", DERIVED)], &[]),
    Case::new("hand-written constructors", &[Source::new("a.rs", CONSTRUCTORS)], &["a.rs:5"]),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}
