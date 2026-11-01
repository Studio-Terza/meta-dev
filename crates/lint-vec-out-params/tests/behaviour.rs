use lint_vec_out_params::VecOutParams;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const INLINE: &str = "mod inner {\n    pub fn fill(out: &mut Vec<u8>) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn fill(out: &mut Vec<u8>) {}\n}\n";

const CASES: [Case; 2] = [
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
];

#[test]
fn vec_out_parameters_are_found_in_inline_modules() {
    assert_eq!(Sources::mismatches::<VecOutParams>(&CASES), Lines::default());
}
