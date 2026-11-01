use lint_primitive_boundary::PrimitiveBoundary;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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

const PUBLIC_STR: &str = "pub fn f(x: &str) {}\n";
const PUBLIC_STRING: &str = "pub fn f() -> String {\n    g()\n}\n";
const PRIVATE_TEXT: &str = "fn f(x: &str) -> String {\n    g()\n}\n";
const CRATE_TEXT: &str = "pub(crate) fn f(x: &str) {}\n";
const NESTED_TEXT: &str = "pub fn f(x: Option<String>, y: Vec<&str>, z: &mut String) {}\n";
const LIFETIME_TEXT: &str = "pub fn f<'a>(x: &'a str) {}\n";
const METHOD_TEXT_PARAM: &str = "pub struct A;\nimpl A {\n    pub fn f(&self, x: &str) {}\n}\n";
const METHOD_PRIVATE_TEXT: &str = "pub struct A;\nimpl A {\n    fn f(&self, x: &str) {}\n}\n";
const FROM_STR: &str = "pub struct A;\nimpl FromStr for A {\n    type Err = E;\n    fn from_str(text: &str) -> Result<A, E> {\n        g()\n    }\n}\n";
const TRAIT_TEXT: &str = "pub trait T {\n    fn f(&self, x: &str) -> String;\n}\n";
const PUBLIC_FIELD: &str = "pub struct A {\n    pub name: String,\n}\n";
const PRIVATE_FIELD: &str = "pub struct A {\n    name: String,\n}\n";
const TUPLE_FIELD: &str = "pub struct A(pub String);\n";
const PRIVATE_TUPLE_FIELD: &str = "pub struct A(String);\n";
const PUBLIC_ENUM: &str = "pub enum E {\n    Named(String),\n}\n";
const PRIVATE_ENUM: &str = "enum E {\n    Named(String),\n}\n";
const TEXT_CONST: &str =
    "pub const NAME: &str = \"a\";\npub static OTHER: &str = \"b\";\npub type Words = Vec<String>;\n";
const TEXT_ASSOCIATED: &str = "pub struct A;\nimpl A {\n    pub const NAME: &str = \"a\";\n}\n";
const TEXT_IN_BODY: &str = "pub fn f() {\n    pub fn g(x: &str) {}\n}\n";
const FOREIGN_PATHS: &str = "pub fn f(x: other::String, y: &other::str) {}\n";
const INLINE_TEXT_MODULE: &str = "mod inner {\n    pub fn f(x: &str) {}\n}\n";
const PORTS_MODULE: &str = "mod ports {\n    pub fn f(x: &str) {}\n}\n";
const TEXT_TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    pub fn f(x: &str) {}\n}\n";
const WIDER_0: &str = "pub fn m(a: std::borrow::Cow<'_, str>) {}\n";
const WIDER_1: &str = "pub struct A {\n    pub a: Box<str>,\n}\n";
const WIDER_2: &str = "pub fn m(a: std::sync::Arc<str>) {}\n";
const WIDER_3: &str = "pub fn m(a: impl AsRef<str>) {}\n";
const WIDER_4: &str = "pub fn m<T>(a: T)\nwhere\n    T: AsRef<str>,\n{\n}\n";
const WIDER_5: &str = "pub fn m(a: *const str) {}\n";
const WIDER_6: &str = "pub fn m(a: std::string::String) {}\n";
const WIDER_7: &str = "pub fn m(a: alloc::string::String) {}\n";
const WIDER_8: &str = "use std::string::String as S;\npub fn m(a: S) {}\n";
const WIDER_9: &str = "pub struct A<T = String> {\n    pub a: T,\n}\n";
const WIDER_10: &str = "union U {\n    a: &'static str,\n}\n";
const WIDER_11: &str = "unsafe extern \"C\" {\n    fn m(a: &str);\n}\n";

const CASES: [Case; 74] = [
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("bits module file"),
        &[Source::new(Located::new("crates/x/src/bits/mask.rs"), Text::new(BARE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("bits module root"),
        &[Source::new(Located::new("crates/x/src/bits.rs"), Text::new(BARE))],
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
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("other module file"),
        &[Source::new(Located::new("crates/x/src/masks/mask.rs"), Text::new(BARE))],
        Flagged::new(&["crates/x/src/masks/mask.rs:1", "crates/x/src/masks/mask.rs:1"]),
    ),
    Case::new(
        Title::new("integration test"),
        &[Source::new(Located::new("x/tests/a.rs"), Text::new(BARE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inherent method"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("trait method"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_METHOD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("enum context"),
        &[Source::new(Located::new("a.rs"), Text::new(ENUM_SIZE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("enum named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(ENUM_OF_LENGTHS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("const context"),
        &[Source::new(Located::new("a.rs"), Text::new(CONST_SIZE))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("const named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(CONST_LENGTH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("impl const context"),
        &[Source::new(Located::new("a.rs"), Text::new(IMPL_CONST))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("impl const named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(IMPL_LENGTH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait const context"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_CONST))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("trait const named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_LENGTH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("alias context"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIAS))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("alias named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIAS_OF_LENGTH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("two fields are no newtype"),
        &[Source::new(Located::new("a.rs"), Text::new(PAIR))],
        Flagged::new(&["a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("String::from"),
        &[Source::new(Located::new("a.rs"), Text::new(STRING_FROM))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(Title::new("Vec::from"), &[Source::new(Located::new("a.rs"), Text::new(VEC_FROM))], Flagged::new(&[])),
    Case::new(
        Title::new("String::new"),
        &[Source::new(Located::new("a.rs"), Text::new(STRING_NEW))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("format and other macros"),
        &[Source::new(Located::new("a.rs"), Text::new(MACROS))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("text in an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_TEXT))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("text in a method"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD_TEXT))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("another two-part path"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_PATH))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a size parameter named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(PARAM_SIZE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a size parameter in a method named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(PARAM_UNNAMED_SIZE))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a size return in a method named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(RETURN_SIZE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a size return in a method not named for a size"),
        &[Source::new(Located::new("a.rs"), Text::new(RETURN_UNNAMED_SIZE))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("text inside a Display impl"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in an inherent impl of a Display type"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR_AND_HELPER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in an impl of another type"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR_AND_OTHER_TYPE))],
        Flagged::new(&["a.rs:10"]),
    ),
    Case::new(
        Title::new("text in a free function beside a Display impl on a reference"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR_ON_A_REFERENCE))],
        Flagged::new(&["a.rs:8"]),
    ),
    Case::new(
        Title::new("a public function taking &str"),
        &[Source::new(Located::new("a.rs"), Text::new(PUBLIC_STR))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a public function returning String"),
        &[Source::new(Located::new("a.rs"), Text::new(PUBLIC_STRING))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a private function with text"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a crate-visible function with text"),
        &[Source::new(Located::new("a.rs"), Text::new(CRATE_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text nested in other types"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED_TEXT))],
        Flagged::new(&["a.rs:1", "a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("text behind a lifetime"),
        &[Source::new(Located::new("a.rs"), Text::new(LIFETIME_TEXT))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a public method taking &str"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD_TEXT_PARAM))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a private method taking &str"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD_PRIVATE_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in a FromStr impl"),
        &[Source::new(Located::new("a.rs"), Text::new(FROM_STR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in a trait declaration"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a public field of String"),
        &[Source::new(Located::new("a.rs"), Text::new(PUBLIC_FIELD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a private field of String"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE_FIELD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a public tuple field of String"),
        &[Source::new(Located::new("a.rs"), Text::new(TUPLE_FIELD))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a private tuple field of String"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE_TUPLE_FIELD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a field of a public enum"),
        &[Source::new(Located::new("a.rs"), Text::new(PUBLIC_ENUM))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a field of a private enum"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE_ENUM))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in constants and aliases"),
        &[Source::new(Located::new("a.rs"), Text::new(TEXT_CONST))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in an associated constant"),
        &[Source::new(Located::new("a.rs"), Text::new(TEXT_ASSOCIATED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in a function body"),
        &[Source::new(Located::new("a.rs"), Text::new(TEXT_IN_BODY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text spelled through another path"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_PATHS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_TEXT_MODULE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("an inline ports module"),
        &[Source::new(Located::new("a.rs"), Text::new(PORTS_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a file in a ports module"),
        &[Source::new(Located::new("crates/x/src/ports/run.rs"), Text::new(PUBLIC_STR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a ports module root"),
        &[Source::new(Located::new("crates/x/src/ports.rs"), Text::new(PUBLIC_STR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a public function in a test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEXT_TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a public function in an integration test"),
        &[Source::new(Located::new("x/tests/a.rs"), Text::new(PUBLIC_STR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text beside a Display impl"),
        &[Source::new(Located::new("a.rs"), Text::new(DOOR_AND_TRIM))],
        Flagged::new(&["a.rs:8"]),
    ),
    Case::new(
        Title::new("text in a Cow"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_0))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("text in a box"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_1))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("text in a shared pointer"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_2))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("text behind AsRef"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_3))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("text behind AsRef in a where clause"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_4))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("text behind a raw pointer"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_5))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a qualified String"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_6))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a String from alloc"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_7))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a renamed String"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_8))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a String as a default type"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_9))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("text in a private union"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_10))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("text in a private extern block"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDER_11))],
        Flagged::new(&[]),
    ),
];

#[test]
fn primitives_away_from_a_types_boundary_are_flagged() {
    assert_eq!(Sources::mismatches::<PrimitiveBoundary>(&CASES), Lines::default());
}
