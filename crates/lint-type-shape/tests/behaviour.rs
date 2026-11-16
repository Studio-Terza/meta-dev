use lint_file::Swept;
use lint_type_shape::TypeShape;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const METHOD: &str = "struct A;\nimpl A {\n    fn pair(&self) -> (u8, u8) { (1, 2) }\n}\n";
const TRAIT_METHOD: &str = "struct A;\nimpl Tr for A {\n    fn pair(&self) -> (u8, u8) { (1, 2) }\n}\n";
const LEFT_ROOT: &str = "struct A {\n    a: u8,\n}\nfn f() {\n    let pair: (u8, u8) = (1, 2);\n}\n";

const CASES: [Case; 3] = [
    Case::new(
        Title::new("inherent method"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("trait method"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_METHOD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a tuple after a field type, outside any checked type"),
        &[Source::new(Located::new("a.rs"), Text::new(LEFT_ROOT))],
        Flagged::new(&[]),
    ),
];

#[test]
fn signatures_of_inherent_methods_are_checked() {
    assert_eq!(Sources::mismatches::<Swept<TypeShape>>(&CASES), Lines::default());
}
