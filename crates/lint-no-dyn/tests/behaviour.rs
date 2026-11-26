use lint_file::Filed;
use lint_no_dyn::NoDyn;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TRAIT_OBJECTS: &str = "trait Shape {}\nfn f(shape: &dyn Shape) {}\nstruct Held(Box<dyn Shape + Send>);\nfn g() -> Box<dyn Fn(u8) -> u8> {\n    todo!()\n}\n";
const IN_MACRO: &str = "wrap! {\n    type Shapes = Vec<Box<dyn Shape>>;\n}\n";
const ONLY_TEXT: &str = "fn f() -> &'static str {\n    \"dyn Shape\"\n}\n// dyn in a comment\n/* dyn */\nfn dynamic(dyns: u8) -> u8 {\n    dyns\n}\n";

const CASES: [Case; 3] = [
    Case::new(
        Title::new("each dyn in a type is flagged on its line"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(TRAIT_OBJECTS))],
        Flagged::new(&["src/lib.rs:2", "src/lib.rs:3", "src/lib.rs:4"]),
    ),
    Case::new(
        Title::new("dyn inside a macro body is code"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["src/lib.rs:2"]),
    ),
    Case::new(
        Title::new("dyn in strings, comments and longer names is not the keyword"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(ONLY_TEXT))],
        Flagged::new(&[]),
    ),
];

#[test]
fn only_the_dyn_keyword_in_code_is_flagged() {
    assert_eq!(Sources::mismatches::<Filed<NoDyn>>(&CASES), Lines::default());
}
