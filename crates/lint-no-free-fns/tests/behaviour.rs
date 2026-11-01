use lint_no_free_fns::NoFreeFns;

use lint_fixtures::{Case, Source, Sources};

const OTHER_ATTRIBUTE: &str = "#[inline]\nfn helper() {}\n";
const IN_MACRO: &str = "macro_rules! helpers {\n    () => {\n        fn helper() {}\n    };\n}\n";

const CASES: [Case; 2] = [
    Case::new("other attribute", &[Source::new("a.rs", OTHER_ATTRIBUTE)], &["a.rs:2"]),
    Case::new("inside a macro", &[Source::new("a.rs", IN_MACRO)], &["a.rs:3"]),
];

#[test]
fn free_functions_stay_flagged_behind_attributes_and_macros() {
    assert_eq!(Sources::mismatches::<NoFreeFns>(&CASES), Vec::<String>::new());
}
