use code_model::{Implementation, Origin, Resolved};
use code_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::Name;

const DERIVED: &str = "#[derive(Clone,\n    Debug)]\nstruct A;\n#[derive(PartialEq)]\nenum B {\n    C,\n}\n#[derive(Copy)]\nunion D {\n    e: u8,\n}\n";
const WRITTEN: &str = "mod m {\n    impl Clone for A {}\n}\nfn f() {\n    impl A {}\n}\n";
const MIXED: &str = "#[derive(Debug)]\nstruct A;\nimpl Clone for A {}\n";

const RENAMED: &str = "use std::{clone::Clone as Dup};\nimpl Dup for R {}\nimpl a::Dup for R {}\n";
const ROOTED: &str = "impl core::fmt::Debug for R {}\nimpl alloc::borrow::ToOwned for R {}\nimpl ::std::hash::Hash for R {}\nimpl ::serde::Serialize for R {}\nimpl crate::a::T for R {}\nimpl self::T for R {}\nimpl super::T for R {}\n";
const BOUND: &str = "use core::fmt;\nuse self::m::T as U;\nuse other::Shown;\nimpl fmt::Debug for R {}\nimpl U for R {}\nimpl Shown for R {}\n";
const GROUPED: &str = "use std::{fmt::{Debug, Display as Shown}, hash::*};\nimpl Debug for R {}\nimpl Shown for R {}\nimpl Hasher for R {}\n";
const DECLARED: &str =
    "trait Clone {}\nimpl Clone for R {}\nimpl Copy for R {}\nimpl Display for R {}\nimpl a::Clone for R {}\n";
const SCOPED: &str = "use std::clone::Clone as Dup;\nmod a {\n    impl Dup for R {}\n}\nfn f() {\n    use std::fmt::Debug as Show;\n    impl Dup for R {}\n    impl Show for R {}\n    mod b {\n        impl Show for R {}\n    }\n}\nimpl Show for R {}\n";
const SHADOWED: &str = "use std::clone::Clone;\nfn f() {\n    trait Clone {}\n    impl Clone for R {}\n}\n";
const DERIVES: &str = "#[derive(Clone, serde::Serialize)]\nstruct A;\n";
const MACRO_SCOPE: &str = "mod m {\n    use std::fmt::Debug as Show;\n    macro_rules! make {\n        () => { impl Show for R {} };\n    }\n}\n";
const MACROS: &str = "macro_rules! make {\n    ($t:ty, $u:ty) => {\n        impl<T> Clone for Wrapper<T> {}\n        impl PartialEq<$u> for $t {}\n        impl $t {\n            fn each() { for x in y {} }\n        }\n        mod inner {\n            impl Inner {}\n            impl Copy for Inner {}\n            impl $tr for Inner {}\n        }\n    };\n}\nfn f() {\n    other! {\n        impl Eq for Skipped {}\n    }\n}\n";

struct Expect;

impl Expect {
    fn written(subject: &str, resolved: Resolved) -> Implementation {
        Implementation::new(Name::from(subject), Some(resolved), Origin::Written)
    }

    fn std(path: &[&str]) -> Resolved {
        Resolved::Std(path.iter().map(|segment| Name::from(*segment)).collect())
    }

    fn krate(path: &[&str]) -> Resolved {
        Resolved::Crate(path.iter().map(|segment| Name::from(*segment)).collect())
    }

    fn external(path: &[&str]) -> Resolved {
        Resolved::External(path.iter().map(|segment| Name::from(*segment)).collect())
    }
}

struct Read;

impl Read {
    fn file(path: &str, text: &str) -> SourceFile {
        match SourceFile::parse(path, SourceText::new(text.to_owned())) {
            Ok(file) => file,
            Err(failure) => unreachable!("the source parses: {failure}"),
        }
    }

    fn sites(files: &[SourceFile]) -> Vec<String> {
        Reader::read_files(files).implementations().map(|found| found.site().to_string()).collect()
    }

    fn implementations(files: &[SourceFile]) -> Vec<Implementation> {
        Reader::read_files(files).implementations().map(|found| found.entity().clone()).collect()
    }

    fn resolved(text: &str) -> Vec<Resolved> {
        Reader::read_files(&[Read::file("a.rs", text)])
            .implementations()
            .filter_map(|found| found.entity().implemented().cloned())
            .collect()
    }

    fn written(files: &[SourceFile]) -> Vec<String> {
        Reader::read_files(files)
            .implementations()
            .filter_map(|found| found.entity().written().map(|_| found.site().to_string()))
            .collect()
    }
}

laws::table!(every_implementation_is_read_at_its_impl_or_derive_path, |text: &&str| Read::sites(&[Read::file("a.rs", text)]), [
    DERIVED => ["a.rs:1", "a.rs:2", "a.rs:4", "a.rs:8"],
    WRITTEN => ["a.rs:2", "a.rs:5"],
    MIXED => ["a.rs:1", "a.rs:3"],
]);

laws::table!(only_impl_blocks_are_written, |text: &&str| Read::written(&[Read::file("a.rs", text)]), [
    DERIVED => Vec::<&str>::new(),
    WRITTEN => ["a.rs:2", "a.rs:5"],
    MIXED => ["a.rs:3"],
]);

#[test]
fn each_implementation_is_placed_in_its_own_file() {
    assert_eq!(
        Read::sites(&[Read::file("a.rs", MIXED), Read::file("b.rs", WRITTEN)]),
        ["a.rs:1", "a.rs:3", "b.rs:2", "b.rs:5"]
    );
}

#[test]
fn a_renamed_trait_is_read_as_its_original() {
    assert_eq!(
        Read::implementations(&[Read::file("a.rs", RENAMED)]),
        [
            Expect::written("R", Expect::std(&["std", "clone", "Clone"])),
            Expect::written("R", Expect::external(&["a", "Dup"])),
        ]
    );
}

#[test]
fn only_macro_rules_bodies_yield_trait_impls_in_macros() {
    assert_eq!(
        Read::implementations(&[Read::file("a.rs", MACROS)]),
        [
            Expect::written("Wrapper", Expect::std(&["std", "clone", "Clone"])),
            Expect::written("t", Expect::std(&["std", "cmp", "PartialEq"])),
            Expect::written("Inner", Expect::std(&["std", "marker", "Copy"])),
            Expect::written("Inner", Expect::external(&[])),
        ]
    );
}

laws::table!(each_trait_path_resolves_through_the_scopes_it_is_written_in, |text: &&str| Read::resolved(text), [
    ROOTED => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::std(&["std", "borrow", "ToOwned"]),
        Expect::std(&["std", "hash", "Hash"]),
        Expect::external(&["serde", "Serialize"]),
        Expect::krate(&["crate", "a", "T"]),
        Expect::krate(&["crate", "T"]),
        Expect::krate(&["crate", "T"]),
    ],
    BOUND => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::krate(&["crate", "m", "T"]),
        Expect::external(&["other", "Shown"]),
    ],
    GROUPED => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::std(&["std", "fmt", "Display"]),
        Expect::external(&["Hasher"]),
    ],
    DECLARED => [
        Expect::krate(&["crate", "Clone"]),
        Expect::std(&["std", "marker", "Copy"]),
        Expect::external(&["Display"]),
        Expect::external(&["a", "Clone"]),
    ],
    SCOPED => [
        Expect::external(&["Dup"]),
        Expect::std(&["std", "clone", "Clone"]),
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::external(&["Show"]),
        Expect::external(&["Show"]),
    ],
    SHADOWED => [Expect::krate(&["crate", "Clone"])],
    DERIVES => [Expect::std(&["std", "clone", "Clone"]), Expect::external(&["serde", "Serialize"])],
    MACRO_SCOPE => [Expect::std(&["std", "fmt", "Debug"])],
]);
