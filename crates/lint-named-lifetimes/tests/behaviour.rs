use lint_named_lifetimes::NamedLifetimes;

use lint_fixtures::{Case, Source, Sources};

const TWO_LETTERS: &str = "struct A<'ab>(&'ab str);\n";
const IN_MACRO: &str = "wrap! {\n    struct A<'a>(&'a str);\n}\n";

const CASES: [Case; 2] = [
    Case::new("two letters", &[Source::new("a.rs", TWO_LETTERS)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", IN_MACRO)], &["a.rs:2"]),
];

#[test]
fn only_one_letter_lifetimes_are_flagged_also_inside_macros() {
    assert_eq!(Sources::mismatches::<NamedLifetimes>(&CASES), Vec::<String>::new());
}
