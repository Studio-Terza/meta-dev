use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_hand_impls::HandImpls;

const WRITTEN: &str = "struct A;\nimpl Clone for A {\n    fn clone(&self) -> A { A }\n}\nimpl std::fmt::Debug for A {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\n";
const DERIVED: &str = "#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]\nstruct A;\n";
const OUTSIDE: &str = "struct A;\nimpl std::fmt::Display for A {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\nimpl From<u8> for A {\n    fn from(_: u8) -> A { A }\n}\nimpl A {}\n";

const NESTED: &str = "mod inner {\n    struct A;\n    impl Default for A {\n        fn default() -> A { A }\n    }\n}\nfn f() {\n    struct B;\n    impl Copy for B {}\n}\n#[cfg(test)]\nmod tests {\n    struct C;\n    impl Eq for C {}\n}\n";
const CROSS: &str = "struct A;\nimpl PartialEq<&str> for A {\n    fn eq(&self, _: &&str) -> bool { true }\n}\nimpl core::hash::Hash for A {\n    fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}\n}\n";

const CASES: [Case; 5] = [
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
];

#[test]
fn every_hand_impl_of_a_derivable_std_trait_is_flagged() {
    assert_eq!(Sources::mismatches::<Queried<HandImpls>>(&CASES), Lines::default());
}
