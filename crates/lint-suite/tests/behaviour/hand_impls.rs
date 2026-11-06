use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_hand_impls::HandImpls;

const WRITTEN: &str = "struct A;\nimpl Clone for A {\n    fn clone(&self) -> A { A }\n}\nimpl std::fmt::Debug for A {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\n";
const DERIVED: &str = "#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]\nstruct A;\n";
const OUTSIDE: &str = "struct A;\nimpl std::fmt::Display for A {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\nimpl From<u8> for A {\n    fn from(_: u8) -> A { A }\n}\nimpl A {}\n";

const NESTED: &str = "mod inner {\n    struct A;\n    impl Default for A {\n        fn default() -> A { A }\n    }\n}\nfn f() {\n    struct B;\n    impl Copy for B {}\n}\n#[cfg(test)]\nmod tests {\n    struct C;\n    impl Eq for C {}\n}\n";
const CROSS: &str = "struct A;\nimpl PartialEq<&str> for A {\n    fn eq(&self, _: &&str) -> bool { true }\n}\nimpl core::hash::Hash for A {\n    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}\n}\n";

const ALIASED: &str = "use std::clone::Clone as Dup;\nstruct R;\nimpl Dup for R {\n    fn clone(&self) -> R { R }\n}\n";
const GROUPED: &str = "use std::{fmt::Debug as Show};\nstruct R;\nimpl Show for R {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\n";
const ALIASED_OUTSIDE: &str = "use std::fmt::Display as Shown;\nstruct R;\nimpl Shown for R {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\n";
const MACRO_BANNED: &str = "macro_rules! make {\n    ($t:ty) => {\n        impl Clone for $t {\n            fn clone(&self) -> $t { *self }\n        }\n    };\n}\n";
const MACRO_OUTSIDE: &str = "macro_rules! make {\n    ($t:ty) => {\n        impl Display for $t {\n            fn fmt(&self, f: &mut Formatter<'_>) -> Result { Ok(()) }\n        }\n    };\n}\n";

const QUALIFIED: &str =
    "use std::clone::Clone as Dup;\nmod a {\n    pub trait Dup {}\n}\nstruct R;\nimpl a::Dup for R {}\n";

const OWN_TRAIT: &str = "mod digest {\n    pub trait Hash {\n        fn digest(&self) -> u64;\n    }\n}\nuse digest::Hash;\nstruct Block;\nimpl Hash for Block {\n    fn digest(&self) -> u64 { 0 }\n}\n";
const DECLARED_HERE: &str = "trait Hash {}\nstruct Block;\nimpl Hash for Block {}\n";
const SCOPED_RENAME: &str = "mod a {\n    use std::clone::Clone as Dup;\n}\nmod b {\n    pub trait Dup {}\n    pub struct R;\n    impl Dup for R {}\n}\n";
const CORE_IMPORT: &str = "use core::fmt::Debug;\nstruct R;\nimpl Debug for R {\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) }\n}\n";
const FULL_PATH: &str =
    "struct R;\nimpl std::hash::Hash for R {\n    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}\n}\n";
const OTHER_CRATE: &str = "struct R;\nimpl other_crate::Clone for R {\n    fn clone(&self) -> R { R }\n}\n";
const BLOCK_IMPORT: &str = "struct R;\nfn f() {\n    use std::clone::Clone as Dup;\n    impl Dup for R {\n        fn clone(&self) -> R { R }\n    }\n}\n";

const CASES: [Case; 18] = [
    Case::new(
        Title::new("written std impls"),
        &[Source::new(Located::new("a.rs"), Text::new(WRITTEN))],
        Flagged::new(&["a.rs:2", "a.rs:5"]),
    ),
    Case::new(
        Title::new("inline modules, function bodies and test modules"),
        &[Source::new(Located::new("a.rs"), Text::new(NESTED))],
        Flagged::new(&["a.rs:3", "a.rs:9", "a.rs:14"]),
    ),
    Case::new(
        Title::new("cross-type and fully qualified impls"),
        &[Source::new(Located::new("a.rs"), Text::new(CROSS))],
        Flagged::new(&["a.rs:2", "a.rs:5"]),
    ),
    Case::new(Title::new("derived"), &[Source::new(Located::new("a.rs"), Text::new(DERIVED))], Flagged::new(&[])),
    Case::new(
        Title::new("traits outside the ban"),
        &[Source::new(Located::new("a.rs"), Text::new(OUTSIDE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a renamed trait import"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a renamed trait in a grouped import"),
        &[Source::new(Located::new("a.rs"), Text::new(GROUPED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a renamed trait outside the ban"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED_OUTSIDE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an impl in a macro_rules body"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_BANNED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("an impl outside the ban in a macro_rules body"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_OUTSIDE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a qualified path to a different trait sharing a rename"),
        &[Source::new(Located::new("a.rs"), Text::new(QUALIFIED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an own trait sharing a derivable name"),
        &[Source::new(Located::new("a.rs"), Text::new(OWN_TRAIT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an own trait declared in the same scope"),
        &[Source::new(Located::new("a.rs"), Text::new(DECLARED_HERE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a rename in a sibling module"),
        &[Source::new(Located::new("a.rs"), Text::new(SCOPED_RENAME))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a trait imported from core"),
        &[Source::new(Located::new("a.rs"), Text::new(CORE_IMPORT))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a full std path"),
        &[Source::new(Located::new("a.rs"), Text::new(FULL_PATH))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a derivable name in another crate"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_CRATE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a rename inside a function body"),
        &[Source::new(Located::new("a.rs"), Text::new(BLOCK_IMPORT))],
        Flagged::new(&["a.rs:4"]),
    ),
];

#[test]
fn every_hand_impl_of_a_derivable_std_trait_is_flagged() {
    assert_eq!(Sources::mismatches::<Queried<HandImpls>>(&CASES), Lines::default());
}
