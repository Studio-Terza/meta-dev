use lint_bool_methods::BoolMethods;
use lint_core::Lint;
use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const LISTED: &str = "pub struct A;\nimpl A {\n    pub fn is_on(&self) -> bool { true }\n    pub fn has_room(&self) -> bool { true }\n}\n";
const NESTED: &str =
    "mod inner {\n    pub struct A;\n    impl A {\n        pub fn is_on(&self) -> bool { true }\n    }\n}\n";
const PRIVATE: &str = "pub struct A;\nimpl A {\n    fn is_on(&self) -> bool { true }\n    pub(crate) fn has_room(&self) -> bool { true }\n}\n";
const OTHER_NAMES: &str = "pub struct A;\nimpl A {\n    pub fn allows(&self) -> bool { true }\n    pub fn island(&self) -> bool { true }\n}\n";
const OTHER_RETURNS: &str =
    "pub struct A;\nimpl A {\n    pub fn is_on(&self) -> Option<u8> { None }\n    pub fn has_room(&self) {}\n}\n";
const TRAIT_IMPL: &str = "pub struct A;\nimpl Tr for A {\n    fn is_on(&self) -> bool { true }\n}\n";
const TEST_MODULE: &str = "pub struct A;\n#[cfg(test)]\nmod tests {\n    use super::A;\n    impl A {\n        pub fn is_on(&self) -> bool { true }\n    }\n}\n";
const TEST_IMPL: &str = "pub struct A;\n#[cfg(test)]\nimpl A {\n    pub fn is_on(&self) -> bool { true }\n}\n";

const PUBLIC_TRAIT: &str =
    "pub trait T {\n    fn is_on(&self) -> bool;\n    fn has_room(&self) -> bool;\n    fn size(&self) -> u8;\n}\n";
const PRIVATE_TRAIT: &str = "trait T {\n    fn is_on(&self) -> bool;\n}\n";
const TEST_TRAIT: &str = "#[cfg(test)]\npub trait T {\n    fn is_on(&self) -> bool;\n}\n";

const CASES: [Case; 12] = [
    Case::new(
        Title::new("is_ and has_"),
        &[Source::new(Located::new("a.rs"), Text::new(LISTED))],
        Flagged::new(&["a.rs:3", "a.rs:4"]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("private methods"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("other names"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_NAMES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("other returns"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_RETURNS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait impls"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test modules"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("test impls"), &[Source::new(Located::new("a.rs"), Text::new(TEST_IMPL))], Flagged::new(&[])),
    Case::new(
        Title::new("test targets"),
        &[Source::new(Located::new("x/tests/a.rs"), Text::new(LISTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("public trait"),
        &[Source::new(Located::new("a.rs"), Text::new(PUBLIC_TRAIT))],
        Flagged::new(&["a.rs:2", "a.rs:3"]),
    ),
    Case::new(
        Title::new("private trait"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE_TRAIT))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("test trait"), &[Source::new(Located::new("a.rs"), Text::new(TEST_TRAIT))], Flagged::new(&[])),
];

#[test]
fn public_boolean_predicates_are_listed() {
    assert_eq!(Sources::mismatches::<BoolMethods>(&CASES), Lines::default());
}

#[test]
fn the_message_names_the_replacements() {
    let shown = BoolMethods::report(&[Sources::file("a.rs", Text::new(LISTED))]).to_string();
    assert!(["Option", "enum", "NonEmpty", "fn is_on -> bool"].iter().all(|word| shown.contains(word)));
}
