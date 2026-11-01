use lint_vec_out_params::VecOutParams;

use lint_fixtures::{Case, Source, Sources};

const INLINE: &str = "mod inner {\n    pub fn fill(out: &mut Vec<u8>) {}\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn fill(out: &mut Vec<u8>) {}\n}\n";

const CASES: [Case; 2] = [
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:2"]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
];

#[test]
fn vec_out_parameters_are_found_in_inline_modules() {
    assert_eq!(Sources::mismatches::<VecOutParams>(&CASES), Vec::<String>::new());
}
