use lint_literal_names::LiteralNames;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn get(&self) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn get() {}\n}\n";

const GENERATED: &str = "struct A;\nmacro_rules! vague {\n    () => {\n        fn get() {}\n    };\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn get() {}\n";

const INLINE: &str = "mod inner {\n    pub fn get() {}\n}\n";
const REQUIRED_METHOD: &str = "trait T {\n    fn get(&self);\n}\n";

const CASES: [Case; 6] = [
    Case::new(Title::new("trait impl"), &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPL))], Flagged::new(&[])),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(Title::new("test fn"), &[Source::new(Located::new("a.rs"), Text::new(TEST_FN))], Flagged::new(&[])),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("trait method"),
        &[Source::new(Located::new("a.rs"), Text::new(REQUIRED_METHOD))],
        Flagged::new(&["a.rs:2"]),
    ),
];

#[test]
fn function_names_that_name_nothing_are_flagged() {
    assert_eq!(Sources::mismatches::<LiteralNames>(&CASES), Lines::default());
}
