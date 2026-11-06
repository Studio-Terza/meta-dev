use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_narrowed_returns::NarrowedReturns;

const FORMS: &str = "struct S;\n#[derive(Debug, thiserror::Error)]\nstruct Failure;\nfn f() -> bool { true }\nfn outer() {\n    fn inner() -> bool { true }\n}\nimpl S {\n    fn check(&self) -> Result<(), u8> { Ok(()) }\n}\ntrait T {\n    fn g(&self) -> Option<Failure>;\n}\n";
const STD_ALIASES: &str = "use std::io;\nfn a() -> std::fmt::Result { Ok(()) }\nfn b() -> core::fmt::Result { Ok(()) }\nfn c() -> io::Result<()> { Ok(()) }\nfn d() -> std::io::Result<()> { Ok(()) }\nfn e() -> io::Result<u8> { Ok(0) }\n";
const CRATE_ALIASES: &str = "struct Failure;\ntype Outcome = Result<(), Failure>;\ntype Checked<T> = Result<T, Failure>;\nfn f() -> Outcome { Ok(()) }\nfn g() -> Checked<()> { Ok(()) }\nfn h() -> Checked<u8> { Ok(0) }\n";
const ERROR_OPTIONS: &str = "struct Failure;\nimpl std::error::Error for Failure {}\nstruct Rejected;\nfn parse() -> Result<u8, Rejected> { Ok(0) }\nfn f() -> Option<Failure> { None }\nfn g() -> Option<Rejected> { None }\nfn h() -> Option<std::io::Error> { None }\n";
const ACROSS_ROOT: &str = "use std::io::Result;\nmod m;\n";
const ACROSS_CHILD: &str = "use super::*;\nfn f() -> Result<()> { Ok(()) }\n";
const INNER_MAIN: &str = "mod inner {\n    fn main() -> Result<(), u8> { Ok(()) }\n}\nfn main() {}\n";

const TRAIT_IMPLS: &str = "use std::fmt;\nstruct A;\nimpl PartialEq for A {\n    fn eq(&self, _: &A) -> bool { true }\n}\nimpl fmt::Display for A {\n    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result { Ok(()) }\n}\n";
const TESTS: &str = "#[test]\nfn t() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }\n#[tokio::test]\nasync fn u() -> Result<(), u8> { Ok(()) }\n";
const ROOT_MAIN: &str = "fn main() -> Result<(), u8> { Ok(()) }\n";
const OTHER_SHAPES: &str = "struct E;\nfn a() -> Result<u8, E> { Ok(0) }\nfn b() -> Option<u8> { None }\nfn c<T>() -> Option<T> { None }\nfn d() -> impl std::future::Future<Output = Result<(), E>> { async { Ok(()) } }\nfn e() -> () {}\n";
const SHADOWED_RESULT: &str =
    "mod m {\n    pub struct Result<A, B>(A, B);\n}\nuse m::Result;\nfn f() -> Result<(), u8> { todo!() }\n";
const OPAQUE_GLOB: &str = "use other::*;\nfn f() -> Result<(), u8> { Ok(()) }\n";
const REJECTED_INPUT: &str = "struct Text;\nstruct X;\nimpl TryFrom<Text> for X {\n    type Error = Text;\n    fn try_from(t: Text) -> Result<X, Text> { Err(t) }\n}\nfn f() -> Option<Text> { None }\n";
const GENERIC_NAMED_LIKE_AN_ERROR: &str =
    "struct Failure;\nfn parse() -> Result<u8, Failure> { Ok(0) }\nfn f<Failure>() -> Option<Failure> { None }\n";

const LIBRARY_MAIN: &str = "pub fn main() -> Result<(), u8> { Ok(()) }\n";
const SELF_OF_AN_ERROR: &str = "struct Failure;\nimpl std::error::Error for Failure {}\nimpl Failure {\n    fn again() -> Option<Self> { None }\n}\n";
const SELF_OF_A_PLAIN_TYPE: &str = "struct Plain;\nimpl Plain {\n    fn again() -> Option<Self> { None }\n}\n";
const DEFAULTED_ERROR: &str = "struct Rejected;\ntype Res<T, E = Rejected> = Result<T, E>;\nfn p() -> Res<u8> { Ok(0) }\nfn q() -> Option<Rejected> { None }\n";
const GIVEN_ERROR: &str = "struct Rejected;\ntype Res<T, E = Rejected> = Result<T, E>;\nfn p() -> Res<u8, bool> { Ok(0) }\nfn q() -> Option<Rejected> { None }\n";
const UNDEFAULTED_ERROR: &str = "struct Rejected;\ntype Res<T, E> = Result<T, E>;\nfn p() -> Res<u8, bool> { Ok(0) }\nfn q() -> Option<Rejected> { None }\n";

const CASES: [Case; 22] = [
    Case::new(
        Title::new("free, nested, inherent and trait declaration functions"),
        &[Source::new(Located::new("a.rs"), Text::new(FORMS))],
        Flagged::new(&["a.rs:4", "a.rs:6", "a.rs:9", "a.rs:12"]),
    ),
    Case::new(
        Title::new("std result aliases"),
        &[Source::new(Located::new("a.rs"), Text::new(STD_ALIASES))],
        Flagged::new(&["a.rs:2", "a.rs:3", "a.rs:4", "a.rs:5"]),
    ),
    Case::new(
        Title::new("crate result aliases"),
        &[Source::new(Located::new("a.rs"), Text::new(CRATE_ALIASES))],
        Flagged::new(&["a.rs:4", "a.rs:5"]),
    ),
    Case::new(
        Title::new("an option of an error type"),
        &[Source::new(Located::new("a.rs"), Text::new(ERROR_OPTIONS))],
        Flagged::new(&["a.rs:5", "a.rs:6", "a.rs:7"]),
    ),
    Case::new(
        Title::new("a result alias imported in another file"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new(ACROSS_ROOT)),
            Source::new(Located::new("src/m.rs"), Text::new(ACROSS_CHILD)),
        ],
        Flagged::new(&["src/m.rs:2"]),
    ),
    Case::new(
        Title::new("a main that is not the crate's main"),
        &[Source::new(Located::new("src/main.rs"), Text::new(INNER_MAIN))],
        Flagged::new(&["src/main.rs:2"]),
    ),
    Case::new(
        Title::new("trait impls keep the trait's signature"),
        &[Source::new(Located::new("a.rs"), Text::new(TRAIT_IMPLS))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("test functions"), &[Source::new(Located::new("a.rs"), Text::new(TESTS))], Flagged::new(&[])),
    Case::new(
        Title::new("the crate's main"),
        &[Source::new(Located::new("src/main.rs"), Text::new(ROOT_MAIN))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("shapes outside the ban"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_SHAPES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a crate type named Result"),
        &[Source::new(Located::new("a.rs"), Text::new(SHADOWED_RESULT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a result behind an opaque glob"),
        &[Source::new(Located::new("a.rs"), Text::new(OPAQUE_GLOB))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the rejected input of TryFrom"),
        &[Source::new(Located::new("a.rs"), Text::new(REJECTED_INPUT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a generic named like an error type"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERIC_NAMED_LIKE_AN_ERROR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a library root's main"),
        &[Source::new(Located::new("src/lib.rs"), Text::new(LIBRARY_MAIN))],
        Flagged::new(&["src/lib.rs:1"]),
    ),
    Case::new(
        Title::new("a binary file listed under bin"),
        &[Source::new(Located::new("src/bin/tool.rs"), Text::new(ROOT_MAIN))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a file no crate root reaches"),
        &[
            Source::new(Located::new("src/lib.rs"), Text::new("")),
            Source::new(Located::new("src/loose.rs"), Text::new(ROOT_MAIN)),
        ],
        Flagged::new(&["src/loose.rs:1"]),
    ),
    Case::new(
        Title::new("an option of Self in an error type's impl"),
        &[Source::new(Located::new("a.rs"), Text::new(SELF_OF_AN_ERROR))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("an option of Self in a plain type's impl"),
        &[Source::new(Located::new("a.rs"), Text::new(SELF_OF_A_PLAIN_TYPE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an alias default error type"),
        &[Source::new(Located::new("a.rs"), Text::new(DEFAULTED_ERROR))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("an alias default overridden at the use"),
        &[Source::new(Located::new("a.rs"), Text::new(GIVEN_ERROR))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an alias without a default"),
        &[Source::new(Located::new("a.rs"), Text::new(UNDEFAULTED_ERROR))],
        Flagged::new(&[]),
    ),
];

#[test]
fn every_narrowed_return_outside_trait_impls_tests_and_main_is_flagged() {
    assert_eq!(Sources::mismatches::<Queried<NarrowedReturns>>(&CASES), Lines::default());
}
