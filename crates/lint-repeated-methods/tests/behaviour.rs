use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Source, Sources};

const THREE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const TWO: &str = "struct A;\nstruct B;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const PRIVATE: &str = "struct A;\nstruct B;\nstruct C;\nimpl A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl C {\n    fn show(&self) -> u8 { 1 }\n}\n";
const TRAIT_IMPLS: &str = "struct A;\nstruct B;\nstruct C;\nimpl Tr for A {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for B {\n    fn show(&self) -> u8 { 1 }\n}\nimpl Tr for C {\n    fn show(&self) -> u8 { 1 }\n}\n";

const DERIVED: &str = "#[derive(Newtype)]\npub struct A(u8);\n#[derive(Newtype)]\npub struct B(u8);\n#[derive(Newtype)]\npub struct C(u8);\n";
const CONSTRUCTORS: &str = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u8) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u8) -> Self { C(inner) }\n}\n";

const MIXED_INNERS: &str = "pub struct A(u8);\npub struct B(u16);\npub struct C { value: u32 }\nimpl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u16) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u32) -> Self { C { value: inner } }\n}\n";
const ARITHMETIC: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { A(self.0 + other.0) }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { B(self.0 + other.0) }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { C(self.0 + other.0) }\n}\n";
const SHOWN_THE_SAME: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\nimpl fmt::Display for B {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\nimpl fmt::Display for C {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"{}\", self.0) }\n}\n";
const SHOWN_DIFFERENTLY: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl fmt::Display for A {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"a{}\", self.0) }\n}\nimpl fmt::Display for B {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"b{}\", self.0) }\n}\nimpl fmt::Display for C {\n    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result { write!(formatter, \"c{}\", self.0) }\n}\n";
const ARITHMETIC_TWICE: &str = "pub struct A(usize);\npub struct B(usize);\npub struct C(usize);\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { A(self.0 + other.0) }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { B(self.0 + other.0) }\n}\n";
const NOT_NEWTYPES: &str = "pub struct A;\npub struct B(u8, u8);\npub struct C { a: u8, b: u8 }\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> A { other }\n}\nimpl Add for B {\n    type Output = B;\n    fn add(self, other: B) -> B { other }\n}\nimpl Add for C {\n    type Output = C;\n    fn add(self, other: C) -> C { other }\n}\n";

const CASES: [Case; 12] = [
    Case::new("three types", &[Source::new("a.rs", THREE)], &["a.rs:5"]),
    Case::new("two types", &[Source::new("a.rs", TWO)], &[]),
    Case::new("private methods", &[Source::new("a.rs", PRIVATE)], &[]),
    Case::new("trait impls", &[Source::new("a.rs", TRAIT_IMPLS)], &[]),
    Case::new("derived newtypes", &[Source::new("a.rs", DERIVED)], &[]),
    Case::new("hand-written constructors", &[Source::new("a.rs", CONSTRUCTORS)], &["a.rs:5"]),
    Case::new("constructors over different inner types", &[Source::new("a.rs", MIXED_INNERS)], &["a.rs:5"]),
    Case::new("hand-written arithmetic", &[Source::new("a.rs", ARITHMETIC)], &["a.rs:6"]),
    Case::new("the same Display body", &[Source::new("a.rs", SHOWN_THE_SAME)], &["a.rs:5"]),
    Case::new("different Display bodies", &[Source::new("a.rs", SHOWN_DIFFERENTLY)], &[]),
    Case::new("arithmetic on two newtypes", &[Source::new("a.rs", ARITHMETIC_TWICE)], &[]),
    Case::new("arithmetic on types that are not newtypes", &[Source::new("a.rs", NOT_NEWTYPES)], &[]),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Vec::<String>::new());
}
