use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Source, Sources};

const THREE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const TWO: &str = "struct A;\nstruct B;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const CASES: [Case; 4] = [
    Case::new("three types", &[Source::new("a.rs", THREE)], &["a.rs:5"]),
    Case::new("two types", &[Source::new("a.rs", TWO)], &[]),
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPLS)], &[]),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}
