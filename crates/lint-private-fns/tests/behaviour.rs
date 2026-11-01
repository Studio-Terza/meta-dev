use lint_private_fns::PrivateFns;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const SPLIT: &str = "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n}\nimpl A {\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";
const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn one(&self) {}\n    fn two(&self) {}\n    fn three(&self) {}\n    fn four(&self) {}\n    fn five(&self) {}\n}\n";

const TEST_MODULE: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    impl super::A {\n        fn one(&self) {}\n        fn two(&self) {}\n        fn three(&self) {}\n        fn four(&self) {}\n        fn five(&self) {}\n    }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! crowd {\n    () => {\n        impl A {\n            fn one(&self) {}\n            fn two(&self) {}\n            fn three(&self) {}\n            fn four(&self) {}\n            fn five(&self) {}\n        }\n    };\n}\n";

const INLINE: &str = "mod inner {\n    struct A;\n    impl A {\n        fn one(&self) {}\n        fn two(&self) {}\n        fn three(&self) {}\n        fn four(&self) {}\n        fn five(&self) {}\n    }\n}\n";

const CASES: [Case; 5] = [
    Case::new(
        Title::new("split blocks"),
        &[Source::new(Located::new("a.rs"), Text::new(SPLIT))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(Title::new("trait impl"), &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPL))], Flagged::new(&[])),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:1"]),
    ),
];

#[test]
fn types_with_many_private_fns_are_flagged() {
    assert_eq!(Sources::mismatches::<PrivateFns>(&CASES), Lines::default());
}
