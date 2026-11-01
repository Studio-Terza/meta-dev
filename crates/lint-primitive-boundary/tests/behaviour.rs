use lint_primitive_boundary::PrimitiveBoundary;

use lint_fixtures::{Case, Source, Sources};

const BARE: &str = "fn f(x: u32) -> f64 {\n    g(x)\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(x: u32) {}\n}\n";

const INLINE_BITS: &str = "mod bits {\n    pub fn f(x: u32) {}\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f(x: u32) {}\n}\n";

const METHOD: &str = "struct A;\nimpl A {\n    fn f(x: u32) {}\n}\n";
const TRAIT_METHOD: &str = "struct A;\nimpl Tr for A {\n    fn f(x: u32) {}\n}\n";
const ENUM_SIZE: &str = "enum Buffer {\n    Full(usize),\n}\n";
const ENUM_OF_LENGTHS: &str = "enum Length {\n    Full(usize),\n}\n";
const CONST_SIZE: &str = "const LIMIT: usize = 1;\n";
const CONST_LENGTH: &str = "const LENGTH: usize = 1;\n";
const IMPL_CONST: &str = "struct A;\nimpl A {\n    const LIMIT: usize = 1;\n}\n";
const IMPL_LENGTH: &str = "struct A;\nimpl A {\n    const LENGTH: usize = 1;\n}\n";
const TRAIT_CONST: &str = "trait T {\n    const LIMIT: usize;\n}\n";
const TRAIT_LENGTH: &str = "trait T {\n    const LENGTH: usize;\n}\n";
const ALIAS: &str = "type Limit = usize;\n";
const ALIAS_OF_LENGTH: &str = "type Length = usize;\n";
const PAIR: &str = "struct Pair(u32, u32);\n";
const STRING_FROM: &str = "fn f() -> Name {\n    String::from(\"a\")\n}\n";
const VEC_FROM: &str = "fn f() -> Items {\n    Vec::from(x)\n}\n";
const STRING_NEW: &str = "fn f() -> Name {\n    String::new()\n}\n";
const MACROS: &str = "fn f() {\n    let t = format!(\"x\");\n    let u = vec!(1);\n}\n";
const INLINE_TEXT: &str = "mod inner {\n    pub fn f(x: Name) {\n        x.trim();\n    }\n}\n";
const METHOD_TEXT: &str = "struct A;\nimpl A {\n    fn f(x: Name) {\n        x.trim();\n    }\n}\n";
const OTHER_PATH: &str = "fn f(x: other::Thing) {}\n";

const PARAM_SIZE: &str = "struct A;\nimpl A {\n    fn at(&self, index: usize) {}\n}\n";
const PARAM_UNNAMED_SIZE: &str = "struct A;\nimpl A {\n    fn len(&self, step: usize) {}\n}\n";
const RETURN_SIZE: &str = "struct A;\nimpl A {\n    fn len(&self) -> usize {\n        1\n    }\n}\n";
const RETURN_UNNAMED_SIZE: &str = "struct A;\nimpl A {\n    fn step(&self) -> usize {\n        1\n    }\n}\n";
const DOOR: &str = "struct A;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\n";
const DOOR_AND_TRIM: &str = "struct A;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nfn g(x: Name) {\n    x.trim();\n}\n";

const DOOR_AND_HELPER: &str = "struct A;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nimpl A {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\n";
const DOOR_AND_OTHER_TYPE: &str = "struct A;\nstruct B;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nimpl B {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\n";

const DOOR_ON_A_REFERENCE: &str = "struct A;\nimpl fmt::Display for &A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nfn g(x: Name) {\n    x.trim();\n}\n";

const CASES: [Case; 36] = [
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &[]),
    Case::new("bits module file", &[Source::new("crates/x/src/bits/mask.rs", BARE)], &[]),
    Case::new("bits module root", &[Source::new("crates/x/src/bits.rs", BARE)], &[]),
    Case::new("inline bits module", &[Source::new("a.rs", INLINE_BITS)], &[]),
    Case::new("inline other module", &[Source::new("a.rs", INLINE_OTHER)], &["a.rs:2"]),
    Case::new(
        "other module file",
        &[Source::new("crates/x/src/masks/mask.rs", BARE)],
        &["crates/x/src/masks/mask.rs:1", "crates/x/src/masks/mask.rs:1"],
    ),
    Case::new("integration test", &[Source::new("x/tests/a.rs", BARE)], &[]),
    Case::new("inherent method", &[Source::new("a.rs", METHOD)], &["a.rs:3"]),
    Case::new("trait method", &[Source::new("a.rs", TRAIT_METHOD)], &[]),
    Case::new("enum context", &[Source::new("a.rs", ENUM_SIZE)], &["a.rs:2"]),
    Case::new("enum named for a size", &[Source::new("a.rs", ENUM_OF_LENGTHS)], &[]),
    Case::new("const context", &[Source::new("a.rs", CONST_SIZE)], &["a.rs:1"]),
    Case::new("const named for a size", &[Source::new("a.rs", CONST_LENGTH)], &[]),
    Case::new("impl const context", &[Source::new("a.rs", IMPL_CONST)], &["a.rs:3"]),
    Case::new("impl const named for a size", &[Source::new("a.rs", IMPL_LENGTH)], &[]),
    Case::new("trait const context", &[Source::new("a.rs", TRAIT_CONST)], &["a.rs:2"]),
    Case::new("trait const named for a size", &[Source::new("a.rs", TRAIT_LENGTH)], &[]),
    Case::new("alias context", &[Source::new("a.rs", ALIAS)], &["a.rs:1"]),
    Case::new("alias named for a size", &[Source::new("a.rs", ALIAS_OF_LENGTH)], &[]),
    Case::new("two fields are no newtype", &[Source::new("a.rs", PAIR)], &["a.rs:1", "a.rs:1"]),
    Case::new("String::from", &[Source::new("a.rs", STRING_FROM)], &["a.rs:2"]),
    Case::new("Vec::from", &[Source::new("a.rs", VEC_FROM)], &[]),
    Case::new("String::new", &[Source::new("a.rs", STRING_NEW)], &[]),
    Case::new("format and other macros", &[Source::new("a.rs", MACROS)], &["a.rs:2"]),
    Case::new("text in an inline module", &[Source::new("a.rs", INLINE_TEXT)], &["a.rs:3"]),
    Case::new("text in a method", &[Source::new("a.rs", METHOD_TEXT)], &["a.rs:4"]),
    Case::new("another two-part path", &[Source::new("a.rs", OTHER_PATH)], &[]),
    Case::new("a size parameter named for a size", &[Source::new("a.rs", PARAM_SIZE)], &[]),
    Case::new("a size parameter in a method named for a size", &[Source::new("a.rs", PARAM_UNNAMED_SIZE)], &["a.rs:3"]),
    Case::new("a size return in a method named for a size", &[Source::new("a.rs", RETURN_SIZE)], &[]),
    Case::new(
        "a size return in a method not named for a size",
        &[Source::new("a.rs", RETURN_UNNAMED_SIZE)],
        &["a.rs:3"],
    ),
    Case::new("text inside a Display impl", &[Source::new("a.rs", DOOR)], &[]),
    Case::new("text in an inherent impl of a Display type", &[Source::new("a.rs", DOOR_AND_HELPER)], &[]),
    Case::new("text in an impl of another type", &[Source::new("a.rs", DOOR_AND_OTHER_TYPE)], &["a.rs:10"]),
    Case::new(
        "text in a free function beside a Display impl on a reference",
        &[Source::new("a.rs", DOOR_ON_A_REFERENCE)],
        &["a.rs:8"],
    ),
    Case::new("text beside a Display impl", &[Source::new("a.rs", DOOR_AND_TRIM)], &["a.rs:8"]),
];

#[test]
fn primitives_away_from_a_types_boundary_are_flagged() {
    assert_eq!(Sources::mismatches::<PrimitiveBoundary>(&CASES), Vec::<String>::new());
}
