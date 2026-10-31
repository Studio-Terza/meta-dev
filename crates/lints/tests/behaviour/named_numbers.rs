use lints::NamedNumbers;

use crate::common::{Case, Source, Sources};

const LITERAL: &str = "fn f() -> u32 {\n    42\n}\n";
const FLOAT: &str = "fn f() -> f64 {\n    2.5\n}\n";
const IN_MACRO: &str = "fn f() {\n    println!(\"{}\", 7);\n}\n";
const NAMED: &str =
    "const LIMIT: u32 = 42;\nstruct A;\nimpl A {\n    const SIZE: u32 = 7;\n}\nconst TABLE: [u8; 3] = [4, 5, 6];\n";
const UNITS: &str = "fn f() -> u32 {\n    0 + 1\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() -> u32 {\n        42\n    }\n}\n";
const SUPPORT_MODULE: &str = "#[cfg(test)]\nmod fixtures;\n";
const STATIC: &str = "static TABLE: [u8; 2] = [4, 5];\n";

const CASES: [Case; 8] = [
    Case { name: "static", sources: &[Source::new("a.rs", STATIC)], flagged: &["a.rs:1", "a.rs:1", "a.rs:1"] },
    Case { name: "integer", sources: &[Source::new("a.rs", LITERAL)], flagged: &["a.rs:2"] },
    Case { name: "float", sources: &[Source::new("a.rs", FLOAT)], flagged: &["a.rs:2"] },
    Case { name: "in a macro", sources: &[Source::new("a.rs", IN_MACRO)], flagged: &["a.rs:2"] },
    Case { name: "named", sources: &[Source::new("a.rs", NAMED)], flagged: &[] },
    Case { name: "units", sources: &[Source::new("a.rs", UNITS)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &[] },
    Case {
        name: "support module",
        sources: &[Source::new("src/lib.rs", SUPPORT_MODULE), Source::new("src/fixtures.rs", LITERAL)],
        flagged: &[],
    },
];

#[test]
fn numbers_outside_const_items_are_flagged() {
    assert_eq!(Sources::mismatches::<NamedNumbers>(&CASES), Vec::<String>::new());
}
