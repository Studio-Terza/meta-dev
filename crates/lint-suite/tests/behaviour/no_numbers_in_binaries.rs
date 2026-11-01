use lint_no_numbers_in_binaries::NoNumbersInBinaries;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const INLINE: &str = "fn main() {}\nmod inner {\n    fn pause() -> u32 {\n        30\n    }\n}\n";

const CASES: [Case; 1] = [Case::new(
    Title::new("inline module"),
    &[Source::new(Located::new("src/main.rs"), Text::new(INLINE))],
    Flagged::new(&["src/main.rs:4"]),
)];

#[test]
fn numbers_in_a_binary_are_flagged() {
    assert_eq!(Sources::mismatches::<NoNumbersInBinaries>(&CASES), Lines::default());
}
