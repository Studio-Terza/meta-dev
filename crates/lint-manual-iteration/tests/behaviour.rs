use lint_manual_iteration::ManualIteration;

use lint_fixtures::{Case, Source, Sources};

const MUTABLE_ITERATOR: &str = "fn f(items: Vec<u8>) {\n    let mut walk = items.iter();\n    walk.next();\n}\n";
const ACCUMULATING_LOOP: &str =
    "fn f(items: Vec<u8>, out: &mut Vec<u8>) {\n    for item in items {\n        out.push(item);\n    }\n}\n";
const FILTERED_LOOP: &str = "fn f(items: Vec<u8>, out: &mut Vec<u8>) {\n    for item in items {\n        if item > 1 {\n            out.push(item);\n        }\n    }\n}\n";
const COUNTER: &str = "fn f() -> u8 {\n    let mut total = 0;\n    total\n}\n";
const WORKING_LOOP: &str = "fn f(items: Vec<u8>) {\n    for item in items {\n        g(item);\n    }\n}\n";
const TEST_MODULE: &str =
    "#[cfg(test)]\nmod tests {\n    fn f(items: Vec<u8>) {\n        let mut walk = items.iter();\n    }\n}\n";

const GENERATED: &str = "macro_rules! walking {\n    () => {\n        fn f(items: Vec<u8>) {\n            let mut walk = items.iter();\n        }\n    };\n}\n";
const TEST_FN: &str = "#[cfg(test)]\nfn f(items: Vec<u8>) {\n    let mut walk = items.iter();\n}\n";

const CASES: [Case; 8] = [
    Case::new("mutable iterator", &[Source::new("a.rs", MUTABLE_ITERATOR)], &["a.rs:2"]),
    Case::new("accumulating loop", &[Source::new("a.rs", ACCUMULATING_LOOP)], &["a.rs:2"]),
    Case::new("filtered loop", &[Source::new("a.rs", FILTERED_LOOP)], &["a.rs:2"]),
    Case::new("counter", &[Source::new("a.rs", COUNTER)], &[]),
    Case::new("working loop", &[Source::new("a.rs", WORKING_LOOP)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("inside a macro", &[Source::new("a.rs", GENERATED)], &["a.rs:4"]),
    Case::new("test fn", &[Source::new("a.rs", TEST_FN)], &[]),
];

#[test]
fn iterators_driven_by_hand_are_flagged() {
    assert_eq!(Sources::mismatches::<ManualIteration>(&CASES), Vec::<String>::new());
}
