use lint_private_fns::PrivateFns;

use lint_fixtures::{Case, Source, Sources};

const CROWDED: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";
const ENOUGH: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n}\n";
const PUBLIC: &str = "struct A;\nimpl A {\n    pub fn one(&self) {}\n    pub fn two(&self) {}\n    pub fn three(&self) {}\n    pub fn four(&self) {}\n    pub fn five(&self) {}\n}\n";
const SPLIT: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n}\nimpl A {\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";
const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";

const TEST_MODULE: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    impl super::A {\n        fn one(&self) {}\n        fn two(&self) {}\n        fn three(&self) {}\n        fn four(&self) {}\n        fn five(&self) {}\n    }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! crowd {\n    () => {\n        impl A {\n            fn one(&self) {}\n            fn two(&self) {}\n            fn three(&self) {}\n            fn four(&self) {}\n            fn five(&self) {}\n        }\n    };\n}\n";

const CASES: [Case; 7] = [
    Case::new("crowded", &[Source::new("a.rs", CROWDED)], &["a.rs:1"]),
    Case::new("enough", &[Source::new("a.rs", ENOUGH)], &[]),
    Case::new("public", &[Source::new("a.rs", PUBLIC)], &[]),
    Case::new("split blocks", &[Source::new("a.rs", SPLIT)], &["a.rs:1"]),
    Case::new("trait impl", &[Source::new("a.rs", TRAIT_IMPL)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:1"]),
];

#[test]
fn types_with_many_private_fns_are_flagged() {
    assert_eq!(Sources::mismatches::<PrivateFns>(&CASES), Vec::<String>::new());
}
