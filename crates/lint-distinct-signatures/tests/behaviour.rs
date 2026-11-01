use lint_distinct_signatures::DistinctSignatures;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TWO_TYPES: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn get(&self) -> u8 { 1 }\n}\nimpl B {\n    fn get(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPL: &str =
    "struct A;\nimpl Tr for A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u8 { 2 }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct A;\n    impl A {\n        fn left(&self) -> u8 { 1 }\n        fn right(&self) -> u8 { 2 }\n    }\n}\n";

const PLATFORMS: &str = "struct A;\nimpl A {\n    #[cfg(unix)]\n    fn read(&self) -> u8 { 1 }\n    #[cfg(windows)]\n    fn read(&self) -> u8 { 1 }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! pair {\n    () => {\n        impl A {\n            fn left(&self) -> u8 { 1 }\n            fn right(&self) -> u8 { 2 }\n        }\n    };\n}\n";

const INLINE: &str = "mod inner {\n    struct A;\n    impl A {\n        fn left(&self) -> u8 { 1 }\n        fn right(&self) -> u8 { 2 }\n    }\n}\n";

const CASES: [Case; 6] = [
    Case::new(Title::new("two types"), &[Source::new(Located::new("a.rs"), Text::new(TWO_TYPES))], Flagged::new(&[])),
    Case::new(Title::new("trait impl"), &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPL))], Flagged::new(&[])),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("platform variants"),
        &[Source::new(Located::new("a.rs"), Text::new(PLATFORMS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:4"]),
    ),
];

#[test]
fn functions_sharing_one_signature_on_one_type_are_flagged() {
    assert_eq!(Sources::mismatches::<DistinctSignatures>(&CASES), Lines::default());
}
