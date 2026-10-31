use lints::LiteralNames;

use crate::common::{Case, Source, Sources};

const VAGUE: &str = "fn get() {}\nstruct A;\nimpl A {\n    fn handle(&self) {}\n    fn go(&self) {}\n}\n";
const SAID: &str = "fn parse_header() {}\nstruct A;\nimpl A {\n    fn render_row(&self) {}\n}\n";
const TRAIT_IMPL: &str = "struct A;\nimpl Tr for A {\n    fn get(&self) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn get() {}\n}\n";

const CASES: [Case; 4] = [
    Case { name: "vague", sources: &[Source::new("a.rs", VAGUE)], flagged: &["a.rs:1", "a.rs:4", "a.rs:5"] },
    Case { name: "said", sources: &[Source::new("a.rs", SAID)], flagged: &[] },
    Case { name: "trait impl", sources: &[Source::new("a.rs", TRAIT_IMPL)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
];

#[test]
fn function_names_that_name_nothing_are_flagged() {
    assert_eq!(Sources::mismatches::<LiteralNames>(&CASES), Vec::<String>::new());
}
