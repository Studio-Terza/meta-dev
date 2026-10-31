use lints::ExpressionFlow;

use crate::common::{Case, Source, Sources};

const EARLY_RETURN: &str = "fn f(x: u8) -> u8 {\n    if x > 1 {\n        return 2;\n    }\n    3\n}\n";
const LOOPS: &str = "fn f() {\n    for _ in [1, 2] {}\n    while true {}\n    loop {}\n}\n";
const DROPPED: &str = "fn f(x: u8) {\n    match x {\n        1 => g(),\n        _ => h(),\n    };\n}\n";
const LONELY_IF: &str = "fn f(x: bool) {\n    if x {\n        g();\n    }\n    h();\n}\n";
const REASSIGNED: &str = "fn f() -> u8 {\n    let mut total = 1;\n    total = 2;\n    total\n}\n";
const COMPOUND: &str = "fn f() -> u8 {\n    let mut total = 1;\n    total += 2;\n    total\n}\n";
const TAIL: &str = "fn f(x: bool) -> u8 {\n    if x { 1 } else { 2 }\n}\n";
const MUTATED: &str = "fn f() -> Vec<u8> {\n    let mut items = Vec::new();\n    items.push(1);\n    items\n}\n";
const CONST_LOOP: &str = "const fn f() -> u8 {\n    let mut i = 0;\n    while i < 3 { i += 1; }\n    i\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() {\n        for _ in [1] {}\n    }\n}\n";

const WRITTEN: &str =
    "fn f() -> String {\n    let mut out = String::new();\n    write!(out, \"x\").ok();\n    out\n}\n";
const CLOSURE: &str =
    "fn f(limit: u8) {\n    let mut seen = Seen::default();\n    let mut bump = || seen.note(limit);\n    bump();\n}\n";
const TEST_IMPL: &str = "struct A;\n#[cfg(test)]\nimpl A {\n    fn f() {\n        for _ in [1] {}\n    }\n}\n";
const GENERATED: &str = "struct A;\nmacro_rules! looping {\n    () => {\n        impl A {\n            fn f() {\n                for _ in [1] {}\n            }\n        }\n    };\n}\n";

const CASES: [Case; 14] = [
    Case {
        name: "early return",
        sources: &[Source::new("a.rs", EARLY_RETURN)],
        flagged: &["a.rs:2", "a.rs:2", "a.rs:3"],
    },
    Case { name: "loops", sources: &[Source::new("a.rs", LOOPS)], flagged: &["a.rs:2", "a.rs:3", "a.rs:4"] },
    Case { name: "dropped match", sources: &[Source::new("a.rs", DROPPED)], flagged: &["a.rs:2"] },
    Case { name: "lonely if", sources: &[Source::new("a.rs", LONELY_IF)], flagged: &["a.rs:2", "a.rs:2"] },
    Case { name: "reassigned", sources: &[Source::new("a.rs", REASSIGNED)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "compound", sources: &[Source::new("a.rs", COMPOUND)], flagged: &["a.rs:2", "a.rs:3"] },
    Case { name: "tail if", sources: &[Source::new("a.rs", TAIL)], flagged: &[] },
    Case { name: "mutated", sources: &[Source::new("a.rs", MUTATED)], flagged: &[] },
    Case { name: "const loop", sources: &[Source::new("a.rs", CONST_LOOP)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
    Case { name: "written by a macro", sources: &[Source::new("a.rs", WRITTEN)], flagged: &[] },
    Case { name: "called closure", sources: &[Source::new("a.rs", CLOSURE)], flagged: &[] },
    Case { name: "test impl", sources: &[Source::new("a.rs", TEST_IMPL)], flagged: &[] },
    Case { name: "inside a macro", sources: &[Source::new("a.rs", GENERATED)], flagged: &["a.rs:6"] },
];

#[test]
fn control_flow_that_is_not_an_expression_is_flagged() {
    assert_eq!(Sources::mismatches::<ExpressionFlow>(&CASES), Vec::<String>::new());
}
