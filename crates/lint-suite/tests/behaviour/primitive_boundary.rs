use lint_primitive_boundary::PrimitiveBoundary;

use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};

const BARE: &str = "fn f(x: u32) -> f64 {\n    g(x)\n}\n";
const SUPPORT_MODULE: &str = "#[cfg(test)]\nmod fixtures;\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    fn f(x: u32) {}\n}\n";
const INLINE_BOUNDARY: &str = "mod boundary {\n    pub fn f(x: u32) -> String {\n        x.to_string()\n    }\n}\n";
const INLINE_OTHER: &str = "mod masks {\n    pub fn f(x: u32) {}\n}\n";
const METHOD: &str = "struct A;\nimpl A {\n    fn f(x: u32) {}\n}\n";
const TRAIT_METHOD: &str = "struct A;\nimpl Tr for A {\n    fn f(x: u32) -> bool {\n        g(x)\n    }\n}\n";
const TRAIT_DECLARATION: &str = "trait T {\n    fn f(&self, x: &str) -> String;\n}\n";
const VARIANT: &str = "enum Buffer {\n    Full(usize),\n    Named { count: u8 },\n}\n";
const CONSTANT: &str = "const LIMIT: usize = 1;\nstatic NAME: &str = \"a\";\n";
const NEWTYPE: &str = "struct Count(usize);\nstruct Label<'text>(&'text str);\n";
const TWO_FIELDS: &str = "struct Pair(u32, u32);\n";
const NAMED_FIELD: &str = "pub struct A {\n    name: String,\n    size: Size,\n}\n";
const NEWTYPE_EDGES: &str = "struct Count(usize);\nimpl Count {\n    const fn new(count: usize) -> Count {\n        Count(count)\n    }\n    fn try_new(count: usize) -> Result<Count, E> {\n        g(count)\n    }\n    fn into_inner(self) -> usize {\n        self.0\n    }\n}\n";
const NEWTYPE_ACCESSOR: &str =
    "struct Count(usize);\nimpl Count {\n    fn count(self) -> usize {\n        self.0\n    }\n}\n";
const CONSTRUCTOR_ELSEWHERE: &str =
    "struct A {\n    b: B,\n}\nimpl A {\n    fn new(x: u32) -> A {\n        g(x)\n    }\n}\n";
const WIDE_CONSTRUCTOR: &str = "struct Count(usize);\nimpl Count {\n    fn new(count: usize, more: usize) -> Count {\n        g(count, more)\n    }\n}\n";
const NESTED: &str = "pub fn f(x: Option<String>, y: Vec<&str>, z: &mut [u8]) {}\n";
const FLAG: &str = "fn f(x: bool) {}\n";
const TUPLE: &str = "fn f(x: (u8, char)) {}\n";
const CLOSURE: &str = "fn f(x: impl Fn(&str) -> bool) {}\n";
const BOUND: &str = "fn m<T>(a: T)\nwhere\n    T: AsRef<str>,\n{\n}\n";
const ALIASED: &str = "type Label = String;\nfn f(x: Label) {}\n";
const ALIASED_QUALIFIED: &str = "type Label = <A>::String;\nfn f(x: Label) {}\n";
const TWO_FIELD_CONSTRUCTOR: &str =
    "struct Pair(A, B);\nimpl Pair {\n    fn new(x: u32) -> Pair {\n        g(x)\n    }\n}\n";
const QUALIFIED: &str = "fn m(a: std::string::String, b: alloc::string::String) {}\n";
const RENAMED: &str = "use std::string::String as S;\nfn m(a: S) {}\n";
const FOREIGN_PATHS: &str = "fn f(x: other::String, y: &other::str, z: <A>::String) {}\n";
const IN_BODY: &str = "fn f() {\n    fn g(x: &str) {}\n}\n";
const BODY_TYPES: &str = "fn f(x: A) -> B {\n    let y: u32 = x.into();\n    h(y.cast::<u64>())\n}\n";
const UNIT: &str = "fn f(x: A) {}\nfn g(x: A) -> Result<(), E> {\n    h(x)\n}\n";
const WIRE: &str = "fn f(x: serde_json::Value) {}\nstruct A {\n    b: toml::Value,\n}\n";
const EXTERN: &str = "unsafe extern \"C\" {\n    fn m(a: &str);\n}\n";
const UNION: &str = "union U {\n    a: &'static str,\n}\n";
const DEFAULTED: &str = "pub struct A<T = String> {\n    pub a: T,\n}\n";
const FROM_STR: &str = "pub struct A;\nimpl FromStr for A {\n    type Err = E;\n    fn from_str(text: &str) -> Result<A, E> {\n        g(text)\n    }\n}\n";
const STRING_FROM: &str = "fn f() -> Name {\n    String::from(\"a\")\n}\n";
const MACROS: &str = "fn f() {\n    let t = format!(\"x\");\n    let u = vec!(1);\n}\n";
const INLINE_TEXT: &str = "mod inner {\n    pub fn f(x: Name) {\n        x.trim();\n    }\n}\n";
const DOOR: &str = "struct A;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\n";
const DOOR_AND_HELPER: &str = "struct A;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nimpl A {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\n";
const DOOR_AND_OTHER_TYPE: &str = "struct A;\nstruct B;\nimpl fmt::Display for A {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"{}\", format!(\"x\"))\n    }\n}\nimpl B {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\n";
const TEST_NEWTYPE: &str = "pub struct Id(u64);\n";
const SHIPPED_CONSTRUCTOR: &str =
    "pub struct Id {\n    v: V,\n}\nimpl Id {\n    pub fn new(v: u64) -> Id {\n        g(v)\n    }\n}\n";
const TEST_DOOR: &str = "struct A;\nimpl A {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\n#[cfg(test)]\nmod tests {\n    impl fmt::Display for super::A {\n        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n            g(f)\n        }\n    }\n}\n";
const MACRO_DOOR: &str = "struct A;\nimpl A {\n    fn g(x: Name) {\n        x.trim();\n    }\n}\nm! {\n    impl fmt::Display for A {\n        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n            g(f)\n        }\n    }\n}\n";
const MACRO_TEXT: &str =
    "fn f(x: Name) {\n    assert!(x.trim().is_empty());\n    println!(\"{}\", x.to_string());\n}\n";
const BESIDE_MACRO: &str = "m! {\n    fn f(x: &str) {}\n}\nfn g(y: u8) {}\n";
const SAME_NAME_DOOR: &str = "mod m {\n    pub struct A;\n    impl std::fmt::Display for A {\n        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n            g(f)\n        }\n    }\n}\nmod n {\n    pub struct A;\n    impl A {\n        fn f(x: Name) {\n            x.trim();\n        }\n    }\n}\n";
const SAME_NAME_NEWTYPE: &str = "pub struct Id(u64);\nmod other {\n    pub struct Id {\n        v: V,\n    }\n    impl Id {\n        pub fn new(v: u64) -> Id {\n            g(v)\n        }\n    }\n}\n";
const ANY_TRAIT: &str = "use std::any::Any;\npub fn f(a: &dyn Any, b: impl core::any::Any) {}\npub fn g<T: std::any::Any>(t: T, c: u32) {}\n";
const MODULE_ALIAS: &str =
    "pub type Count = usize;\nmod inner {\n    use super::Count;\n    fn c(x: Count) {}\n}\nfn d(x: Count) {}\n";

const CASES: [Case; 56] = [
    Case::new(
        Title::new("std's Any is a trait, so dyn, impl and bound uses of it are not primitives"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(ANY_TRAIT))],
        Flagged::new(&["crates/x/src/lib.rs:3"]),
    ),
    Case::new(
        Title::new("a door on another same-named type exempts nothing"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SAME_NAME_DOOR))],
        Flagged::new(&["crates/x/src/lib.rs:13"]),
    ),
    Case::new(
        Title::new("a constructor on a same-named type elsewhere than the newtype"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SAME_NAME_NEWTYPE))],
        Flagged::new(&["crates/x/src/lib.rs:7"]),
    ),
    Case::new(
        Title::new("an alias resolves through module scope"),
        &[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(MODULE_ALIAS))],
        Flagged::new(&["crates/x/src/lib.rs:4", "crates/x/src/lib.rs:6"]),
    ),
    Case::new(
        Title::new("a shipped constructor beside a same-named newtype in a test file"),
        &[
            Source::new(Located::new("crates/x/tests/a.rs"), Text::new(TEST_NEWTYPE)),
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SHIPPED_CONSTRUCTOR)),
        ],
        Flagged::new(&["crates/x/src/lib.rs:5"]),
    ),
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("support module beside a plain file"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(SUPPORT_MODULE)),
            Source::new(Located::new("crates/x/src/fixtures.rs"), Text::new(BARE)),
            Source::new(Located::new("crates/x/src/clock.rs"), Text::new(BARE)),
        ],
        Flagged::new(&["crates/x/src/clock.rs:1", "crates/x/src/clock.rs:1"]),
    ),
    Case::new(
        Title::new("boundary module file"),
        &[Source::new(Located::new("crates/x/src/boundary/mask.rs"), Text::new(BARE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("boundary module root"),
        &[Source::new(Located::new("crates/x/src/boundary.rs"), Text::new(BARE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline boundary module, signatures and text"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_BOUNDARY))],
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
        Title::new("private inherent method"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("trait impl"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_METHOD))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait declaration"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_DECLARATION))],
        Flagged::new(&["a.rs:2", "a.rs:2"]),
    ),
    Case::new(
        Title::new("enum variant fields"),
        &[Source::new(Located::new("a.rs"), Text::new(VARIANT))],
        Flagged::new(&["a.rs:2", "a.rs:3"]),
    ),
    Case::new(
        Title::new("constants and statics"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTANT))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("newtypes"), &[Source::new(Located::new("a.rs"), Text::new(NEWTYPE))], Flagged::new(&[])),
    Case::new(
        Title::new("two fields are no newtype"),
        &[Source::new(Located::new("a.rs"), Text::new(TWO_FIELDS))],
        Flagged::new(&["a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("a private named field"),
        &[Source::new(Located::new("a.rs"), Text::new(NAMED_FIELD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a newtype's own new, try_new and into_inner"),
        &[Source::new(Located::new("a.rs"), Text::new(NEWTYPE_EDGES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("any other newtype accessor"),
        &[Source::new(Located::new("a.rs"), Text::new(NEWTYPE_ACCESSOR))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("new on a type that is not a newtype"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTRUCTOR_ELSEWHERE))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("a newtype constructor with two inputs"),
        &[Source::new(Located::new("a.rs"), Text::new(WIDE_CONSTRUCTOR))],
        Flagged::new(&["a.rs:3", "a.rs:3"]),
    ),
    Case::new(
        Title::new("primitives nested in other types"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED))],
        Flagged::new(&["a.rs:1", "a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("a bool parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(FLAG))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(
        Title::new("a tuple of primitives"),
        &[Source::new(Located::new("a.rs"), Text::new(TUPLE))],
        Flagged::new(&["a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("a closure in a signature"),
        &[Source::new(Located::new("a.rs"), Text::new(CLOSURE))],
        Flagged::new(&["a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("a where bound"),
        &[Source::new(Located::new("a.rs"), Text::new(BOUND))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("an alias of a primitive is read through"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("an alias of a qualified path is not read through"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED_QUALIFIED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("new on a tuple struct of two fields"),
        &[Source::new(Located::new("a.rs"), Text::new(TWO_FIELD_CONSTRUCTOR))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("qualified String"),
        &[Source::new(Located::new("a.rs"), Text::new(QUALIFIED))],
        Flagged::new(&["a.rs:1", "a.rs:1"]),
    ),
    Case::new(
        Title::new("a renamed import"),
        &[Source::new(Located::new("a.rs"), Text::new(RENAMED))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("foreign and qualified paths"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_PATHS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a function inside a body"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_BODY))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("types inside a body"),
        &[Source::new(Located::new("a.rs"), Text::new(BODY_TYPES))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("unit returns"), &[Source::new(Located::new("a.rs"), Text::new(UNIT))], Flagged::new(&[])),
    Case::new(
        Title::new("wire forms"),
        &[Source::new(Located::new("a.rs"), Text::new(WIRE))],
        Flagged::new(&["a.rs:1", "a.rs:3"]),
    ),
    Case::new(
        Title::new("an extern function"),
        &[Source::new(Located::new("a.rs"), Text::new(EXTERN))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a union field"),
        &[Source::new(Located::new("a.rs"), Text::new(UNION))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a defaulted generic"),
        &[Source::new(Located::new("a.rs"), Text::new(DEFAULTED))],
        Flagged::new(&["a.rs:1"]),
    ),
    Case::new(Title::new("FromStr"), &[Source::new(Located::new("a.rs"), Text::new(FROM_STR))], Flagged::new(&[])),
    Case::new(
        Title::new("String::from"),
        &[Source::new(Located::new("a.rs"), Text::new(STRING_FROM))],
        Flagged::new(&["a.rs:2"]),
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
        Title::new("text in a boundary module file"),
        &[Source::new(Located::new("crates/x/src/boundary/parse.rs"), Text::new(INLINE_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a ports module is no boundary"),
        &[Source::new(Located::new("crates/x/src/ports.rs"), Text::new(FLAG))],
        Flagged::new(&["crates/x/src/ports.rs:1"]),
    ),
    Case::new(
        Title::new("a door impl in a cfg(test) module opens no door for shipped text"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_DOOR))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("a door impl in a macro body opens no door for shipped text"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_DOOR))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("text in std macro arguments is not blocked yet"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_TEXT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a primitive beside an item macro"),
        &[Source::new(Located::new("a.rs"), Text::new(BESIDE_MACRO))],
        Flagged::new(&["a.rs:4"]),
    ),
];

#[test]
fn primitives_outside_a_boundary_are_flagged() {
    assert_eq!(Sources::mismatches::<Queried<PrimitiveBoundary>>(&CASES), Lines::default());
}
