use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Source, Sources};

const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const DERIVED: &str = "#[derive(Newtype)]\npub struct A(u8);\n#[derive(Newtype)]\npub struct B(u8);\n#[derive(Newtype)]\npub struct C(u8);\n";
const CONSTRUCTORS: &str = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u8) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u8) -> Self { C(inner) }\n}\n";

const OWN_TYPE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn join(&self, other: A) -> A { other }\n}\nimpl B {\n    pub fn join(&self, other: B) -> B { other }\n}\nimpl C {\n    pub fn join(&self, other: C) -> C { other }\n}\n";
const DIFFERENT_RETURNS: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u16 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u32 { 1 }\n}\n";
const INLINE: &str = "mod inner {\n    struct A;\n    struct B;\n    struct C;\n    impl A {\n        pub fn show(&self) -> u8 { 1 }\n    }\n    impl B {\n        pub fn show(&self) -> u8 { 1 }\n    }\n    impl C {\n        pub fn show(&self) -> u8 { 1 }\n    }\n}\n";

const MIXED_INNERS: &str = "pub struct A(u8);\npub struct B(u16);\npub struct C { value: u32 }\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u16) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u32) -> Self { C { value: inner } }\n}\n";
const ARITHMETIC: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { A(self.0 + other.0) }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { B(self.0 + other.0) }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { C(self.0 + other.0) }\n}\n";
const SHOWN_THE_SAME: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\nimpl fmt::Display for B {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\nimpl fmt::Display for C {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\n";
const SHOWN_DIFFERENTLY: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"a{}\", self.0) }\n}\nimpl fmt::Display for B {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"b{}\", self.0) }\n}\nimpl fmt::Display for C {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"c{}\", self.0) }\n}\n";
const ARITHMETIC_TWICE: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { A(self.0 + other.0) }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { B(self.0 + other.0) }\n}\n";
const NOT_NEWTYPES: &str = "pub struct A;\npub struct B(u8, u8);\npub struct C { a: u8, b: u8 }\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { other }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { other }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { other }\n}\n";

const TWO_FIELDS_NAMED: &str = "pub struct A { a: u8, b: u8 }\npub struct B { a: u8, b: u8 }\npub struct C { a: u8, b: u8 }\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { other }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { other }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { other }\n}\n";
const TWO_FIELDS_TUPLE: &str = "pub struct A(u8, u8);\npub struct B(u8, u8);\npub struct C(u8, u8);\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { other }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { other }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { other }\n}\n";
const OTHER_TRAIT: &str = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\nimpl Tr for A {\n    fn go(self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn go(self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn go(self) -> u8 { 1 }\n}\n";
const DIFFERENT_PARAMS: &str = "pub struct A;\npub struct B;\npub struct C;\nimpl A {\n    pub fn f(&self, x: Foo) {}\n}\nimpl B {\n    pub fn f(&self, x: Bar) {}\n}\nimpl C {\n    pub fn f(&self, x: Baz) {}\n}\n";

const SHOW_ON_A: &str = "struct A;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const SHOW_ON_B: &str = "struct B;\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const WRAPPED_A: Source = Source::new("crates/x/src/a.rs", "pub struct A(u8);\n");
const WRAPPED_B: Source = Source::new("crates/x/src/b.rs", "pub struct B(u16);\n");
const WRAPPED_C: Source = Source::new("crates/x/src/c.rs", "pub struct C(u32);\n");
const CONSTRUCTORS_ELSEWHERE: &str = "impl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u16) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u32) -> Self { C(inner) }\n}\n";

const CASES: [Case; 20] = [
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPLS)], &[]),
    Case::new("derived newtypes", &[Source::new("a.rs", DERIVED)], &[]),
    Case::new("hand-written constructors", &[Source::new("a.rs", CONSTRUCTORS)], &["a.rs:5"]),
    Case::new("own type erased", &[Source::new("a.rs", OWN_TYPE)], &["a.rs:5"]),
    Case::new("different returns", &[Source::new("a.rs", DIFFERENT_RETURNS)], &[]),
    Case::new("inline module", &[Source::new("a.rs", INLINE)], &["a.rs:6"]),
    Case::new("constructors over different inner types", &[Source::new("a.rs", MIXED_INNERS)], &["a.rs:5"]),
    Case::new("hand-written arithmetic", &[Source::new("a.rs", ARITHMETIC)], &["a.rs:6"]),
    Case::new("the same Display body", &[Source::new("a.rs", SHOWN_THE_SAME)], &["a.rs:5"]),
    Case::new("different Display bodies", &[Source::new("a.rs", SHOWN_DIFFERENTLY)], &[]),
    Case::new("arithmetic on two newtypes", &[Source::new("a.rs", ARITHMETIC_TWICE)], &[]),
    Case::new("arithmetic on types that are not newtypes", &[Source::new("a.rs", NOT_NEWTYPES)], &[]),
    Case::new("arithmetic on named structs of two fields", &[Source::new("a.rs", TWO_FIELDS_NAMED)], &[]),
    Case::new("arithmetic on tuple structs of two fields", &[Source::new("a.rs", TWO_FIELDS_TUPLE)], &[]),
    Case::new("the same body of a trait outside the list", &[Source::new("a.rs", OTHER_TRAIT)], &[]),
    Case::new("methods that differ in their parameters", &[Source::new("a.rs", DIFFERENT_PARAMS)], &[]),
    Case::new(
        "two types of one name and a third",
        &[Source::new("a.rs", SHOW_ON_A), Source::new("b.rs", SHOW_ON_B), Source::new("c.rs", SHOW_ON_A)],
        &["a.rs:3"],
    ),
    Case::new(
        "constructors in another file of the crate",
        &[WRAPPED_A, WRAPPED_B, WRAPPED_C, Source::new("crates/x/src/impls.rs", CONSTRUCTORS_ELSEWHERE)],
        &["crates/x/src/impls.rs:2"],
    ),
    Case::new(
        "constructors in another crate",
        &[WRAPPED_A, WRAPPED_B, WRAPPED_C, Source::new("crates/y/src/lib.rs", CONSTRUCTORS_ELSEWHERE)],
        &["crates/y/src/lib.rs:2"],
    ),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}

#[test]
fn a_trait_impl_message_names_the_trait_and_an_inherent_one_does_not() {
    let arithmetic = Sources::report::<RepeatedMethods>(&[Source::new("a.rs", ARITHMETIC)]).to_string();
    let constructors = Sources::report::<RepeatedMethods>(&[Source::new("a.rs", CONSTRUCTORS)]).to_string();
    assert!(arithmetic.contains("in impl Add on A, B, C"));
    assert!(!constructors.contains("in impl"));
}
