use lint_literal_names::LiteralNames;

use lint_fixtures::{Case, Source, Sources};

const VAGUE: &str = "fn get() {}\nstruct A;\nimpl A {\n    fn handle(&self) {}\n    fn go(&self) {}\n}\n";
const SAID: &str = "fn parse_header() {}\nstruct A;\nimpl A {\n    fn render_row(&self) {}\n}\n";
const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn get(&self) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn get() {}\n}\n";

const GENERATED: &str = "struct A;\nmacro_rules! vague {\n    () => {\n        fn get() {}\n    };\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn get() {}\n";

const CASES: [Case; 6] = [
    Case::new("vague", &[Source::new("a.rs", VAGUE)], &["a.rs:1", "a.rs:4", "a.rs:5"]),
    Case::new("said", &[Source::new("a.rs", SAID)], &[]),
    Case::new("trait impl", &[Source::new("a.rs", TRAIT_IMPL)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:4"]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
];

#[test]
fn function_names_that_name_nothing_are_flagged() {
    assert_eq!(Sources::mismatches::<LiteralNames>(&CASES), Vec::<String>::new());
}
