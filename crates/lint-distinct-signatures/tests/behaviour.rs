use lint_distinct_signatures::DistinctSignatures;

use lint_fixtures::{Case, Source, Sources};

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
    Case::new("same signature", &[Source::new("a.rs", SAME)], &["a.rs:3"]),
    Case::new("different return", &[Source::new("a.rs", DIFFERENT)], &[]),
    Case::new("two types", &[Source::new("a.rs", TWO_TYPES)], &[]),
    Case::new("trait impl", &[Source::new("a.rs", TRAIT_IMPL)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("platform variants", &[Source::new("a.rs", PLATFORMS)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:5"]),
];

#[test]
fn functions_sharing_one_signature_on_one_type_are_flagged() {
    assert_eq!(Sources::mismatches::<DistinctSignatures>(&CASES), Vec::<String>::new());
}
