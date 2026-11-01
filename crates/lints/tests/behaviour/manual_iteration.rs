use lints::ManualIteration;

use crate::common::{Case, Source, Sources};

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
    Case { name: "mutable iterator", sources: &[Source::new("a.rs", MUTABLE_ITERATOR)], flagged: &["a.rs:2"] },
    Case { name: "accumulating loop", sources: &[Source::new("a.rs", ACCUMULATING_LOOP)], flagged: &["a.rs:2"] },
    Case { name: "filtered loop", sources: &[Source::new("a.rs", FILTERED_LOOP)], flagged: &["a.rs:2"] },
    Case { name: "counter", sources: &[Source::new("a.rs", COUNTER)], flagged: &[] },
    Case { name: "working loop", sources: &[Source::new("a.rs", WORKING_LOOP)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
    Case { name: "inside a macro", sources: &[Source::new("a.rs", GENERATED)], flagged: &["a.rs:4"] },
    Case { name: "test fn", sources: &[Source::new("a.rs", TEST_FN)], flagged: &[] },
];

#[test]
fn iterators_driven_by_hand_are_flagged() {
    assert_eq!(Sources::mismatches::<ManualIteration>(&CASES), Vec::<String>::new());
}
