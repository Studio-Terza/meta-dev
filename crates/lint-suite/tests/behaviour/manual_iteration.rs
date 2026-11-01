use lint_manual_iteration::ManualIteration;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    fn f(items: Vec<u8>) {\n        let mut walk = items.iter();\n    }\n}\n";

const GENERATED: &str = "macro_rules! walking {\n    () => {\n        fn f(items: Vec<u8>) {\n            let mut walk = items.iter();\n        }\n    };\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn f(items: Vec<u8>) {\n    let mut walk = items.iter();\n}\n";

const INLINE: &str = "mod inner {\n    pub fn f(items: Vec<u8>) {\n        let mut walk = items.iter();\n    }\n}\n";
const METHOD: &str =
    "struct A;\nimpl A {\n    fn f(items: Vec<u8>) {\n        let mut walk = items.iter();\n    }\n}\n";
const DEFAULT_METHOD: &str = "trait T {\n    fn f(items: Vec<u8>) {\n        let mut walk = items.iter();\n    }\n}\n";
const BOTH_BRANCHES: &str = "fn f(items: Vec<u8>, out: &mut Vec<u8>) {\n    for item in items {\n        if item > 1 {\n            out.push(item);\n        } else {\n            out.push(0);\n        }\n    }\n}\n";
const OTHER_METHOD: &str =
    "fn f(items: Vec<u8>, out: &mut Vec<u8>) {\n    for item in items {\n        out.note(item);\n    }\n}\n";
const COMPUTED_RECEIVER: &str =
    "fn f(items: Vec<u8>, out: &mut Vec<u8>) {\n    for item in items {\n        out().push(item);\n    }\n}\n";

const CASES: [Case; 9] = [
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inside a macro"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERATED))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(Title::new("test fn"), &[Source::new(Located::new("a.rs"), Text::new(TEST_FN))], Flagged::new(&[])),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(Title::new("method"), &[Source::new(Located::new("a.rs"), Text::new(METHOD))], Flagged::new(&["a.rs:4"])),
    Case::new(
        Title::new("default method"),
        &[Source::new(Located::new("a.rs"), Text::new(DEFAULT_METHOD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("both branches"),
        &[Source::new(Located::new("a.rs"), Text::new(BOTH_BRANCHES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("other method"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_METHOD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("computed receiver"),
        &[Source::new(Located::new("a.rs"), Text::new(COMPUTED_RECEIVER))],
        Flagged::new(&[]),
    ),
];

#[test]
fn iterators_driven_by_hand_are_flagged() {
    assert_eq!(Sources::mismatches::<ManualIteration>(&CASES), Lines::default());
}
