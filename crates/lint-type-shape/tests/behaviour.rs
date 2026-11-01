use lint_type_shape::TypeShape;

use lint_fixtures::{Case, Source, Sources};

const METHOD: &str = "struct A;\nimpl A {\n    fn pair(&self) -> (u8, u8) { (1, 2) }\n}\n";
const TRAIT_METHOD: &str = "struct A;\nimpl Tr for A {\n    fn pair(&self) -> (u8, u8) { (1, 2) }\n}\n";

const CASES: [Case; 2] = [
    Case::new("inherent method", &[Source::new("a.rs", METHOD)], &["a.rs:3"]),
    Case::new("trait method", &[Source::new("a.rs", TRAIT_METHOD)], &[]),
];

#[test]
fn signatures_of_inherent_methods_are_checked() {
    assert_eq!(Sources::mismatches::<TypeShape>(&CASES), Vec::<String>::new());
}
