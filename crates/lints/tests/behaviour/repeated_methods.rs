use lints::RepeatedMethods;

use crate::common::{Case, Source, Sources};

const THREE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const TWO: &str = "struct A;\nstruct B;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const CASES: [Case; 4] = [
    Case { name: "three types", sources: &[Source::new("a.rs", THREE)], flagged: &["a.rs:5"] },
    Case { name: "two types", sources: &[Source::new("a.rs", TWO)], flagged: &[] },
    Case { name: "private methods", sources: &[Source::new("a.rs", PRIVATE)], flagged: &[] },
    Case { name: "trait impls", sources: &[Source::new("a.rs", TRAIT_IMPLS)], flagged: &[] },
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}
