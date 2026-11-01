use lint_manual_iteration::ManualIteration;

use lint_fixtures::{Case, Source, Sources};

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
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:4"]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:3"]),
    Case::new("method", &[Source::new("a.rs", METHOD)], &["a.rs:4"]),
    Case::new("default method", &[Source::new("a.rs", DEFAULT_METHOD)], &["a.rs:3"]),
    Case::new("both branches", &[Source::new("a.rs", BOTH_BRANCHES)], &[]),
    Case::new("other method", &[Source::new("a.rs", OTHER_METHOD)], &[]),
    Case::new("computed receiver", &[Source::new("a.rs", COMPUTED_RECEIVER)], &[]),
];

#[test]
fn iterators_driven_by_hand_are_flagged() {
    assert_eq!(Sources::mismatches::<ManualIteration>(&CASES), Vec::<String>::new());
}
