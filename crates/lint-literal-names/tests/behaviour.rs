use lint_literal_names::LiteralNames;

use lint_fixtures::{Case, Source, Sources};

const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn get(&self) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn get() {}\n}\n";

const GENERATED: &str = "struct A;\nmacro_rules! vague {\n    () => {\n        fn get() {}\n    };\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn get() {}\n";

const INLINE: &str = "mod inner {\n    pub fn get() {}\n}\n";
const REQUIRED_METHOD: &str = "trait T {\n    fn get(&self);\n}\n";

const CASES: [Case; 6] = [
    Case::new("trait impl", &[Source::new("a.rs", TRAIT_IMPL)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:4"]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:2"]),
    Case::new("trait method", &[Source::new("a.rs", REQUIRED_METHOD)], &["a.rs:2"]),
];

#[test]
fn function_names_that_name_nothing_are_flagged() {
    assert_eq!(Sources::mismatches::<LiteralNames>(&CASES), Vec::<String>::new());
}
