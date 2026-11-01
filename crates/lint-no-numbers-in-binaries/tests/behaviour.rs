use lint_no_numbers_in_binaries::NoNumbersInBinaries;

use lint_fixtures::{Case, Source, Sources};

const INLINE: &str = "fn main() {}\nmod inner {\n    fn pause() -> u32 {\n        30\n    }\n}\n";

const CASES: [Case; 1] = [Case::new("inline module", &[Source::new("src/main.rs", INLINE)], &["src/main.rs:4"])];

#[test]
fn numbers_in_a_binary_are_flagged() {
    assert_eq!(Sources::mismatches::<NoNumbersInBinaries>(&CASES), Vec::<String>::new());
}
