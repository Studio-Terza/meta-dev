use lints::PrivateFns;

use crate::common::{Case, Source, Sources};

const CROWDED: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";
const ENOUGH: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n}\n";
const PUBLIC: &str = "struct A;\nimpl A {\n    pub fn one(&self) {}\n    pub fn two(&self) {}\n    pub fn three(&self) {}\n    pub fn four(&self) {}\n    pub fn five(&self) {}\n}\n";
const SPLIT: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n}\nimpl A {\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";
const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";

const TEST_MODULE: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    impl super::A {\n        fn one(&self) {}\n        fn two(&self) {}\n        fn three(&self) {}\n        fn four(&self) {}\n        fn five(&self) {}\n    }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! crowd {\n    () => {\n        impl A {\n            fn one(&self) {}\n            fn two(&self) {}\n            fn three(&self) {}\n            fn four(&self) {}\n            fn five(&self) {}\n        }\n    };\n}\n";

const CASES: [Case; 7] = [
    Case { name: "crowded", sources: &[Source::new("a.rs", CROWDED)], flagged: &["a.rs:1"] },
    Case { name: "enough", sources: &[Source::new("a.rs", ENOUGH)], flagged: &[] },
    Case { name: "public", sources: &[Source::new("a.rs", PUBLIC)], flagged: &[] },
    Case { name: "split blocks", sources: &[Source::new("a.rs", SPLIT)], flagged: &["a.rs:1"] },
    Case { name: "trait impl", sources: &[Source::new("a.rs", TRAIT_IMPL)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
    Case { name: "inside a macro", sources: &[Source::new("a.rs", GENERATED)], flagged: &["a.rs:1"] },
];

#[test]
fn types_with_many_private_fns_are_flagged() {
    assert_eq!(Sources::mismatches::<PrivateFns>(&CASES), Vec::<String>::new());
}
