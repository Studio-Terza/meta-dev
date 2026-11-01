use lint_named_numbers::NamedNumbers;

use lint_fixtures::{Case, Source, Sources};

const LITERAL: &str = "fn f() -> u32 {\n    42\n}\n";
const FLOAT: &str = "fn f() -> f64 {\n    2.5\n}\n";
const IN_MACRO: &str = "fn f() {\n    println!(\"{}\", 7);\n}\n";
const NAMED: &str =
    "const LIMIT: u32 = 42;\nstruct A;\nimpl A {\n    const SIZE: u32 = 7;\n}\nconst TABLE: [u8; 3] = [4, 5, 6];\n";
const UNITS: &str = "fn f() -> u32 {\n    0 + 1\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() -> u32 {\n        42\n    }\n}\n";
const SUPPORT_MODULE: &str = "#[cfg(test)]\nmod fixtures;\n";
const STATIC: &str = "static TABLE: [u8; 2] = [4, 5];\n";

const FEATURE_SUPPORT: &str = "struct Clock;\n#[cfg(any(test, feature = \"proptest\"))]\nimpl Clock {\n    pub fn range() -> u8 {\n        60\n    }\n}\n";

const INLINE_BITS: &str = "mod bits {\n    pub fn f() -> u32 {\n        42\n    }\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f() -> u32 {\n        42\n    }\n}\n";

const CASES: [Case; 14] = [
    Case::new("static", &[Source::new("a.rs", STATIC)], &["a.rs:1", "a.rs:1", "a.rs:1"]),
    Case::new("integer", &[Source::new("a.rs", LITERAL)], &["a.rs:2"]),
    Case::new("float", &[Source::new("a.rs", FLOAT)], &["a.rs:2"]),
    Case::new("in a macro", &[Source::new("a.rs", IN_MACRO)], &["a.rs:2"]),
    Case::new("named", &[Source::new("a.rs", NAMED)], &[]),
    Case::new("units", &[Source::new("a.rs", UNITS)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new(
        "support module",
        &[Source::new("src/lib.rs", SUPPORT_MODULE), Source::new("src/fixtures.rs", LITERAL)],
        &[],
    ),
    Case::new("test or feature", &[Source::new("a.rs", FEATURE_SUPPORT)], &[]),
    Case::new("bits module file", &[Source::new("crates/x/src/bits/mask.rs", LITERAL)], &[]),
    Case::new("bits module root", &[Source::new("crates/x/src/bits.rs", LITERAL)], &[]),
    Case::new("inline bits module", &[Source::new("a.rs", INLINE_BITS)], &[]),
    Case::new("inline other module", &[Source::new("a.rs", INLINE_OTHER)], &["a.rs:3"]),
    Case::new(
        "other module file",
        &[Source::new("crates/x/src/masks/mask.rs", LITERAL)],
        &["crates/x/src/masks/mask.rs:2"],
    ),
];

#[test]
fn numbers_outside_const_items_are_flagged() {
    assert_eq!(Sources::mismatches::<NamedNumbers>(&CASES), Vec::<String>::new());
}
