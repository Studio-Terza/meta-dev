use lint_named_lifetimes::NamedLifetimes;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TWO_LETTERS: &str = "struct A<'ab>(&'ab str);\n";
const IN_MACRO: &str = "wrap! {\n    struct A<'a>(&'a str);\n}\n";

const CASES: [Case; 2] = [
    Case::new(
        Title::new("two letters"),
        &[Source::new(Located::new("a.rs"), Text::new(TWO_LETTERS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["a.rs:2"]),
    ),
];

#[test]
fn only_one_letter_lifetimes_are_flagged_also_inside_macros() {
    assert_eq!(Sources::mismatches::<NamedLifetimes>(&CASES), Lines::default());
}
