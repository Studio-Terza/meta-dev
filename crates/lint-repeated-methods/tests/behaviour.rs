use lint_repeated_methods::RepeatedMethods;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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

const DIFFERENT_PARAMS: &str = "pub struct A;\npub struct B;\npub struct C;\nimpl A {\n    pub fn f(&self, x: Foo) {}\n}\nimpl B {\n    pub fn f(&self, x: Bar) {}\n}\nimpl C {\n    pub fn f(&self, x: Baz) {}\n}\n";

const SHOW_ON_A: &str = "struct A;\nimpl A {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const SHOW_ON_B: &str = "struct B;\nimpl B {\n    pub fn show(&self) -> u8 { 1 }\n}\n";
const WRAPPED_A: Source = Source::new(Located::new("crates/x/src/a.rs"), Text::new("pub struct A(u8);\n"));
const WRAPPED_B: Source = Source::new(Located::new("crates/x/src/b.rs"), Text::new("pub struct B(u16);\n"));
const WRAPPED_C: Source = Source::new(Located::new("crates/x/src/c.rs"), Text::new("pub struct C(u32);\n"));
const CONSTRUCTORS_ELSEWHERE: &str = "impl A {\n    pub fn new(inner: u8) -> Self { A(inner) }\n}\nimpl B {\n    pub fn new(inner: u16) -> Self { B(inner) }\n}\nimpl C {\n    pub fn new(inner: u32) -> Self { C(inner) }\n}\n";

const CASES: [Case; 14] = [
    Case::new(
        Title::new("private methods"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIVATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("trait impls"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPLS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("derived newtypes"),
        &[Source::new(Located::new("a.rs"), Text::new(DERIVED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("hand-written constructors"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTRUCTORS))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("own type erased"),
        &[Source::new(Located::new("a.rs"), Text::new(OWN_TYPE))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("different returns"),
        &[Source::new(Located::new("a.rs"), Text::new(DIFFERENT_RETURNS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("constructors over different inner types"),
        &[Source::new(Located::new("a.rs"), Text::new(MIXED_INNERS))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("hand-written arithmetic is a standard trait impl"),
        &[Source::new(Located::new("a.rs"), Text::new(ARITHMETIC))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same Display body is a standard trait impl"),
        &[Source::new(Located::new("a.rs"), Text::new(SHOWN_THE_SAME))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("methods that differ in their parameters"),
        &[Source::new(Located::new("a.rs"), Text::new(DIFFERENT_PARAMS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("two types of one name and a third"),
        &[
            Source::new(Located::new("a.rs"), Text::new(SHOW_ON_A)),
            Source::new(Located::new("b.rs"), Text::new(SHOW_ON_B)),
            Source::new(Located::new("c.rs"), Text::new(SHOW_ON_A)),
        ],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("constructors in another file of the crate"),
        &[
            WRAPPED_A,
            WRAPPED_B,
            WRAPPED_C,
            Source::new(Located::new("crates/x/src/impls.rs"), Text::new(CONSTRUCTORS_ELSEWHERE)),
        ],
        Flagged::new(&["crates/x/src/impls.rs:2"]),
    ),
    Case::new(
        Title::new("constructors in another crate"),
        &[
            WRAPPED_A,
            WRAPPED_B,
            WRAPPED_C,
            Source::new(Located::new("crates/y/src/lib.rs"), Text::new(CONSTRUCTORS_ELSEWHERE)),
        ],
        Flagged::new(&["crates/y/src/lib.rs:2"]),
    ),
];

#[test]
fn one_method_shape_on_many_types_is_flagged() {
    assert_eq!(Sources::mismatches::<RepeatedMethods>(&CASES), Lines::default());
}

#[test]
fn a_report_names_the_method_and_the_types_that_repeat_it() {
    let constructors =
        Sources::report::<RepeatedMethods>(&[Source::new(Located::new("a.rs"), Text::new(CONSTRUCTORS))]).to_string();
    assert!(constructors.contains("fn new(Inner) -> Self on A, B, C"));
}
