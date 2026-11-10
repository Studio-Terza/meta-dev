use lint_no_free_fns::NoFreeFns;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const OTHER_ATTRIBUTE: &str = "#[inline]\nfn helper() {}\n";
const IN_MACRO: &str = "macro_rules! helpers {\n    () => {\n        fn helper() {}\n    };\n}\n";

const CASES: [Case; 2] = [
    Case::new(
        Title::new("other attribute"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_ATTRIBUTE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["a.rs:3"]),
    ),
];

#[test]
fn free_functions_stay_flagged_behind_attributes_and_macros() {
    assert_eq!(Sources::mismatches::<NoFreeFns>(&CASES), Lines::default());
}
