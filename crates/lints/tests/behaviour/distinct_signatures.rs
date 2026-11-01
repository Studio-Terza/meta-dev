use lints::DistinctSignatures;

use crate::common::{Case, Source, Sources};

const SAME: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u8 { 2 }\n}\n";
const DIFFERENT: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u16 { 2 }\n}\n";
const TWO_TYPES: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn get(&self) -> u8 { 1 }\n}\nimpl B {\n    fn get(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPL: &str =
    "struct A;\nimpl Tr for A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u8 { 2 }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct A;\n    impl A {\n        fn left(&self) -> u8 { 1 }\n        fn right(&self) -> u8 { 2 }\n    }\n}\n";

const PLATFORMS: &str = "struct A;\nimpl A {\n    #[cfg(unix)]\n    fn read(&self) -> u8 { 1 }\n    #[cfg(windows)]\n    fn read(&self) -> u8 { 1 }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! pair {\n    () => {\n        impl A {\n            fn left(&self) -> u8 { 1 }\n            fn right(&self) -> u8 { 2 }\n        }\n    };\n}\n";

const CASES: [Case; 7] = [
    Case { name: "same signature", sources: &[Source::new("a.rs", SAME)], flagged: &["a.rs:3"] },
    Case { name: "different return", sources: &[Source::new("a.rs", DIFFERENT)], flagged: &[] },
    Case { name: "two types", sources: &[Source::new("a.rs", TWO_TYPES)], flagged: &[] },
    Case { name: "trait impl", sources: &[Source::new("a.rs", TRAIT_IMPL)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
    Case { name: "platform variants", sources: &[Source::new("a.rs", PLATFORMS)], flagged: &[] },
    Case { name: "inside a macro", sources: &[Source::new("a.rs", GENERATED)], flagged: &["a.rs:5"] },
];

#[test]
fn functions_sharing_one_signature_on_one_type_are_flagged() {
    assert_eq!(Sources::mismatches::<DistinctSignatures>(&CASES), Vec::<String>::new());
}
