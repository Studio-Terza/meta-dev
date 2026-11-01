use lint_named_numbers::NamedNumbers;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const LITERAL: &str = "fn f() -> u32 {\n    42\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f() -> u32 {\n        42\n    }\n}\n";
const SUPPORT_MODULE: &str = "#[cfg(test)]\nmod fixtures;\n";
const STATIC: &str = "static TABLE: [u8; 2] = [4, 5];\n";

const FEATURE_SUPPORT: &str = "struct Clock;\n#[cfg(any(test, feature = \"proptest\"))]\nimpl Clock {\n    pub fn range() -> u8 {\n        60\n    }\n}\n";

const INLINE_BITS: &str = "mod bits {\n    pub fn f() -> u32 {\n        42\n    }\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f() -> u32 {\n        42\n    }\n}\n";

const METHOD: &str = "struct A;\nimpl A {\n    fn f() -> u32 {\n        42\n    }\n}\n";
const PLAIN_MODULE: &str = "mod fixtures;\n";

const NEGATIVE_PATTERN: &str =
    "fn f(x: i32) -> bool {\n    match x {\n        -1 => true,\n        _ => false,\n    }\n}\n";
const NEGATIVE_EXPRESSION: &str = "fn f() -> i32 {\n    -1\n}\n";
const NEGATIVE_PATTERN_OTHER: &str =
    "fn f(x: i32) -> bool {\n    match x {\n        -7 => true,\n        _ => false,\n    }\n}\n";
const TEST_ONLY_ALL: &str = "#[cfg(all(test, unix))]\nmod tests {\n    fn f() -> u32 {\n        42\n    }\n}\n";
const FEATURE_ONLY: &str =
    "#[cfg(feature = \"proptest\")]\nmod support {\n    pub fn f() -> u32 {\n        42\n    }\n}\n";

const CASES: [Case; 17] = [
    Case::new(
        Title::new("static"),
        &[Source::new(Located::new("a.rs"), Text::new(STATIC))],
        Flagged::new(&["a.rs:1", "a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("support module"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new(SUPPORT_MODULE)),
            Source::new(Located::new("src/fixtures.rs"), Text::new(LITERAL)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test or feature ships"),
        &[Source::new(Located::new("a.rs"), Text::new(FEATURE_SUPPORT))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("test and platform"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_ONLY_ALL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("feature only ships"),
        &[Source::new(Located::new("a.rs"), Text::new(FEATURE_ONLY))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("negative unit in a pattern"),
        &[Source::new(Located::new("a.rs"), Text::new(NEGATIVE_PATTERN))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("negative unit in an expression"),
        &[Source::new(Located::new("a.rs"), Text::new(NEGATIVE_EXPRESSION))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("negative number in a pattern"),
        &[Source::new(Located::new("a.rs"), Text::new(NEGATIVE_PATTERN_OTHER))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("bits module file"),
        &[Source::new(Located::new("crates/x/src/bits/mask.rs"), Text::new(LITERAL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("bits module root"),
        &[Source::new(Located::new("crates/x/src/bits.rs"), Text::new(LITERAL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline bits module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_BITS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline other module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_OTHER))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("other module file"),
        &[Source::new(Located::new("crates/x/src/masks/mask.rs"), Text::new(LITERAL))],
        Flagged::new(&["crates/x/src/masks/mask.rs:2"]),
    ),
    Case::new(Title::new("method"), &[Source::new(Located::new("a.rs"), Text::new(METHOD))], Flagged::new(&["a.rs:4"])),
    Case::new(
        Title::new("support module of a module"),
        &[
            Source::new(Located::new("src/a.rs"), Text::new(SUPPORT_MODULE)),
            Source::new(Located::new("src/a/fixtures.rs"), Text::new(LITERAL)),
        ],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("plain module"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new(PLAIN_MODULE)),
            Source::new(Located::new("src/fixtures.rs"), Text::new(LITERAL)),
        ],
        Flagged::new(&["src/fixtures.rs:2"]),
    ),
];

#[test]
fn numbers_outside_const_items_are_flagged() {
    assert_eq!(Sources::mismatches::<NamedNumbers>(&CASES), Lines::default());
}
