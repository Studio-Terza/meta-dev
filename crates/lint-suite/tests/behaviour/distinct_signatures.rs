use lint_core::Lint;
use lint_distinct_signatures::DistinctSignatures;
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

use crate::shown::OnGraph;

const SAME: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u8 { 2 }\n}\n";
const DIFFERENT: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u16 { 2 }\n}\n";
const TWO_TYPES: &str =
    "struct A;\nstruct B;\nimpl A {\n    fn get(&self) -> u8 { 1 }\n}\nimpl B {\n    fn get(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPL: &str =
    "struct A;\nimpl Tr for A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> u8 { 2 }\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    struct A;\n    impl A {\n        fn left(&self) -> u8 { 1 }\n        fn right(&self) -> u8 { 2 }\n    }\n}\n";
const PLATFORMS: &str = "struct A;\nimpl A {\n    #[cfg(unix)]\n    fn read(&self) -> u8 { 1 }\n    #[cfg(windows)]\n    fn read(&self) -> u8 { 1 }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! pair {\n    () => {\n        impl A {\n            fn left(&self) -> u8 { 1 }\n            fn right(&self) -> u8 { 2 }\n        }\n    };\n}\n";
const INLINE: &str = "mod inner {\n    struct A;\n    impl A {\n        fn left(&self) -> u8 { 1 }\n        fn right(&self) -> u8 { 2 }\n    }\n}\n";
const DECLARED: &str = "pub mod other;\npub struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n}\n";
const ELSEWHERE: &str = "use crate::A;\nimpl A {\n    fn right(&self) -> u8 { 2 }\n}\n";
const OTHER_CRATE: &str = "struct A;\nimpl A {\n    fn right(&self) -> u8 { 2 }\n}\n";
const TWICE_NAMED: &str = "struct A;\nimpl A {\n    fn right(&self) -> u8 { 2 }\n}\n";
const BORROWED: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(self) -> u8 { 2 }\n}\n";
const PARTIAL: &str =
    "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> Option<u8> { None }\n}\n";
const MANY: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&self) -> impl Iterator<Item = u8> { [2].into_iter() }\n}\n";
const MUTATING: &str = "struct A;\nimpl A {\n    fn left(&self) -> u8 { 1 }\n    fn right(&mut self) -> u8 { 2 }\n}\n";
const OPERATIONS: &str =
    "struct A;\nimpl A {\n    fn join(self, other: A) -> A { self }\n    fn merge(&self, other: &A) -> A { A }\n}\n";

const CASES: [Case; 15] = [
    Case::new(
        Title::new("same signature"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SAME))],
        Flagged::new(&["crates/x/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("different output"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(DIFFERENT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("two types"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(TWO_TYPES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait impl"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(TRAIT_IMPL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("platform variants"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PLATFORMS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(GENERATED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(INLINE))],
        Flagged::new(&["crates/x/src/lib.rs:4"]),
    ),
    Case::new(
        Title::new("impl blocks in two files of one crate"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(DECLARED)),
            Source::new(Located::new("crates/x/src/other.rs"), Text::new(ELSEWHERE)),
        ],
        Flagged::new(&["crates/x/src/lib.rs:4"]),
    ),
    Case::new(
        Title::new("one name in two crates"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(DECLARED)),
            Source::new(Located::new("crates/y/src/lib.rs"), Text::new(OTHER_CRATE)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one name declared twice in one crate"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(DECLARED)),
            Source::new(Located::new("crates/x/src/other.rs"), Text::new(TWICE_NAMED)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("borrowed and owned receiver"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(BORROWED))],
        Flagged::new(&["crates/x/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("plain and optional output"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(PARTIAL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one and many"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(MANY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("mutating and reading"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(MUTATING))],
        Flagged::new(&[]),
    ),
];

#[test]
fn methods_of_one_type_sharing_one_signature_anywhere_in_its_crate_are_flagged() {
    assert_eq!(Sources::mismatches::<OnGraph<DistinctSignatures>>(&CASES), Lines::default());
}

#[test]
fn two_operations_of_one_type_that_read_alike_through_references_are_flagged() {
    let text = OnGraph::<DistinctSignatures>::report(&Sources::parsed(&[Source::new(
        Located::new("crates/x/src/lib.rs"),
        Text::new(OPERATIONS),
    )]))
    .to_string();
    assert!(
        text.contains("crates/x/src/lib.rs:3: x::A: join at crates/x/src/lib.rs:3, merge at crates/x/src/lib.rs:4 share the signature (x::A, x::A) -> x::A; "),
        "{text}"
    );
}
