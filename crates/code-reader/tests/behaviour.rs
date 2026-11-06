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

const GLOB_DECLARED: &str = "trait Clone {}\nmod tests {\n    use super::*;\n    impl Clone for R {}\n}\n";
const GLOB_CRATE: &str = "use crate::shapes::*;\nimpl Hash for R {}\n";
const GLOB_STD: &str =
    "use std::fmt::*;\nuse core::hash::*;\nimpl Debug for R {}\nimpl Hash for R {}\nimpl Clone for R {}\n";
const SELF_IMPORT: &str = "use std::fmt::{self, Write};\nimpl fmt::Debug for R {}\n";
const GLOB_OTHER: &str = "use other::*;\nimpl Clone for R {}\n";
const GLOB_UNLISTED: &str = "use std::collections::*;\nimpl Clone for R {}\n";
const GLOB_SHADOWING: &str = "use std::fmt::*;\nimpl Result for R {}\n";
const GLOB_ENUM: &str = "enum E {\n    A,\n}\nuse E::*;\nimpl Clone for R {}\n";
const GLOB_ITSELF: &str =
    "mod m {\n    pub use std::clone::Clone as Dup;\n}\nuse m::*;\nuse self::m::*;\nimpl Dup for R {}\n";

const MACRO_CALL: &str = "make!();\nimpl Clone for R {}\n";
const MACRO_STATEMENT: &str = "fn f() {\n    make!();\n    impl Clone for R {}\n}\n";
const MACRO_DEFINED: &str = "macro_rules! make {\n    () => {};\n}\nimpl Clone for R {}\n";
const BLOCK_CHAIN: &str = "fn f() {\n    use std::fmt::Debug as Show;\n    {\n        impl Show for R {}\n    }\n    mod m {\n        impl Show for R {}\n    }\n}\n";
const BLOCK_SUPER: &str = "use std::clone::Clone as Dup;\nfn f() {\n    use std::fmt::Debug as Dup;\n    mod m {\n        impl super::Dup for R {}\n    }\n}\n";
const CFG_ALTERNATIVES: &str = "#[cfg(unix)]\ntrait Clone {}\n#[cfg(windows)]\ntrait Clone {}\nimpl Clone for R {}\n";
const ALTERNATIVE_IMPORTS: &str =
    "#[cfg(unix)]\nuse std::clone::Clone as Dup;\n#[cfg(windows)]\nuse std::fmt::Debug as Dup;\nimpl Dup for R {}\n";
const DISCARDED: &str =
    "use std::fmt::Debug as _;\nuse std::fmt::{self as shown};\nimpl Debug for R {}\nimpl shown::Debug for R {}\n";
const ITEM_PATH: &str = "struct S;\nimpl S::T for R {}\nimpl Option::T for R {}\nimpl ::a::T for R {}\n";

const RENAMED_ACROSS: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "pub use std::clone::Clone as Dup;\nmod x;\n"),
    Fixture::new("src/x.rs", "use crate::Dup;\nimpl Dup for R {}\n"),
];
const PARENT_GLOB: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "mod x;\nstruct R;\n"),
    Fixture::new("src/x.rs", "use super::*;\nimpl Clone for R {}\n"),
];
const SIBLING_GLOB: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod shapes;\nmod order;\n"),
    Fixture::new("src/shapes.rs", "pub trait Shape {}\n"),
    Fixture::new("src/order.rs", "use super::shapes::*;\nimpl PartialEq for R {}\n"),
];
const PARENT_DECLARES: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "trait Clone {}\nmod x;\n"),
    Fixture::new("src/x.rs", "use super::*;\nimpl Clone for R {}\n"),
];
const PRIVATE_IMPORT: [Fixture; 4] = [
    Fixture::new("src/lib.rs", "mod p;\nmod q;\n"),
    Fixture::new("src/p.rs", "use std::hash::Hash;\nmod child;\n"),
    Fixture::new("src/p/child.rs", "use super::*;\nimpl Hash for R {}\n"),
    Fixture::new("src/q.rs", "use crate::p::*;\nimpl Hash for R {}\n"),
];
const REEXPORTED: [Fixture; 4] = [
    Fixture::new("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
    Fixture::new("src/a.rs", "pub use std::fmt::Debug as Shown;\n"),
    Fixture::new("src/b.rs", "pub use crate::a::Shown;\nuse crate::a::Shown as Hidden;\n"),
    Fixture::new("src/c.rs", "use crate::b::Shown;\nimpl Shown for R {}\nimpl crate::b::Hidden for R {}\n"),
];
const CYCLE: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod a;\nmod b;\n"),
    Fixture::new("src/a.rs", "pub use crate::b::X;\n"),
    Fixture::new("src/b.rs", "pub use crate::a::X;\nimpl X for R {}\n"),
];
const WORKSPACE: [Fixture; 2] = [
    Fixture::new("crates/a/src/lib.rs", "pub use std::clone::Clone as Dup;\n"),
    Fixture::new("crates/b/src/lib.rs", "impl a::Dup for R {}\nimpl ::a::Dup for R {}\n"),
];
const ISLAND: [Fixture; 1] = [Fixture::new("src/x.rs", "trait T {}\nimpl crate::T for R {}\nimpl super::T for R {}\n")];
const PATHED: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "#[path = \"y.rs\"]\nmod x;\nimpl x::T for R {}\n"),
    Fixture::new("src/x.rs", "pub trait T {}\n"),
];
const NESTED_FILES: [Fixture; 5] = [
    Fixture::new("src/lib.rs", "pub trait Mark {}\nmod a;\nmod c;\n"),
    Fixture::new("src/a/mod.rs", "mod b;\n"),
    Fixture::new("src/a/b.rs", "impl super::super::Mark for R {}\n"),
    Fixture::new("src/c.rs", "mod d {\n    mod e;\n}\n"),
    Fixture::new("src/c/d/e.rs", "impl crate::Mark for R {}\n"),
];
const TWO_CANDIDATES: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod x;\nimpl x::T for R {}\n"),
    Fixture::new("src/x.rs", "pub trait T {}\n"),
    Fixture::new("src/x/mod.rs", "pub trait T {}\n"),
];

struct Fixture {
    path: &'static str,
    text: &'static str,
}

impl Fixture {
    const fn new(path: &'static str, text: &'static str) -> Fixture {
        Fixture { path, text }
    }
}

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

    fn unknown(path: &[&str]) -> Resolved {
        Resolved::Unknown(path.iter().map(|segment| Name::from(*segment)).collect())
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

    fn across(fixtures: &[Fixture]) -> Vec<Resolved> {
        let files: Vec<SourceFile> = fixtures.iter().map(|fixture| Read::file(fixture.path, fixture.text)).collect();
        Reader::read_files(&files).implementations().filter_map(|found| found.entity().implemented().cloned()).collect()
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

laws::table!(each_trait_path_resolves_through_the_scopes_it_is_written_in, |text: &&str| Read::resolved(text), [
    ROOTED => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::std(&["std", "borrow", "ToOwned"]),
        Expect::std(&["std", "hash", "Hash"]),
        Expect::external(&["serde", "Serialize"]),
        Expect::unknown(&["crate", "a", "T"]),
        Expect::unknown(&["self", "T"]),
        Expect::unknown(&["super", "T"]),
    ],
    BOUND => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::unknown(&["U"]),
        Expect::external(&["other", "Shown"]),
    ],
    GROUPED => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::std(&["std", "fmt", "Display"]),
        Expect::std(&["std", "hash", "Hasher"]),
    ],
    DECLARED => [
        Expect::krate(&["crate", "Clone"]),
        Expect::std(&["std", "marker", "Copy"]),
        Expect::unknown(&["Display"]),
        Expect::external(&["a", "Clone"]),
    ],
    SCOPED => [
        Expect::unknown(&["Dup"]),
        Expect::std(&["std", "clone", "Clone"]),
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::unknown(&["Show"]),
        Expect::unknown(&["Show"]),
    ],
    SHADOWED => [Expect::krate(&["crate", "Clone"])],
    DERIVES => [Expect::std(&["std", "clone", "Clone"]), Expect::external(&["serde", "Serialize"])],
    BLOCK_CHAIN => [Expect::std(&["std", "fmt", "Debug"]), Expect::unknown(&["Show"])],
    BLOCK_SUPER => [Expect::std(&["std", "clone", "Clone"])],
    CFG_ALTERNATIVES => [Expect::unknown(&["Clone"])],
    ALTERNATIVE_IMPORTS => [Expect::unknown(&["Dup"])],
    DISCARDED => [Expect::unknown(&["Debug"]), Expect::std(&["std", "fmt", "Debug"])],
    ITEM_PATH => [Expect::unknown(&["S", "T"]), Expect::external(&["Option", "T"]), Expect::external(&["a", "T"])],
]);

laws::table!(a_glob_import_decides_what_a_bare_trait_name_means, |text: &&str| Read::resolved(text), [
    GLOB_DECLARED => [Expect::krate(&["crate", "Clone"])],
    GLOB_CRATE => [Expect::unknown(&["Hash"])],
    GLOB_STD => [
        Expect::std(&["std", "fmt", "Debug"]),
        Expect::std(&["std", "hash", "Hash"]),
        Expect::std(&["std", "clone", "Clone"]),
    ],
    SELF_IMPORT => [Expect::std(&["std", "fmt", "Debug"])],
    GLOB_OTHER => [Expect::unknown(&["Clone"])],
    GLOB_UNLISTED => [Expect::unknown(&["Clone"])],
    GLOB_SHADOWING => [Expect::std(&["std", "fmt", "Result"])],
    GLOB_ENUM => [Expect::unknown(&["Clone"])],
    GLOB_ITSELF => [Expect::std(&["std", "clone", "Clone"])],
]);

laws::table!(an_item_position_macro_hides_what_a_bare_name_means, |text: &&str| Read::resolved(text), [
    MACRO_CALL => [Expect::unknown(&["Clone"])],
    MACRO_STATEMENT => [Expect::unknown(&["Clone"])],
    MACRO_DEFINED => [Expect::std(&["std", "clone", "Clone"])],
]);

laws::table!(a_path_resolves_across_the_files_of_its_crate, Read::across, [
    RENAMED_ACROSS => [Expect::std(&["std", "clone", "Clone"])],
    PARENT_GLOB => [Expect::std(&["std", "clone", "Clone"])],
    SIBLING_GLOB => [Expect::std(&["std", "cmp", "PartialEq"])],
    PARENT_DECLARES => [Expect::krate(&["crate", "Clone"])],
    PRIVATE_IMPORT => [Expect::std(&["std", "hash", "Hash"]), Expect::unknown(&["Hash"])],
    REEXPORTED => [Expect::std(&["std", "fmt", "Debug"]), Expect::unknown(&["crate", "b", "Hidden"])],
    CYCLE => [Expect::unknown(&["X"])],
    WORKSPACE => [Expect::std(&["std", "clone", "Clone"]), Expect::std(&["std", "clone", "Clone"])],
    NESTED_FILES => [Expect::krate(&["crate", "Mark"]), Expect::krate(&["crate", "Mark"])],
]);

laws::table!(a_path_through_an_unplanted_module_is_unknown, Read::across, [
    ISLAND => [Expect::unknown(&["crate", "T"]), Expect::unknown(&["super", "T"])],
    PATHED => [Expect::unknown(&["x", "T"])],
    TWO_CANDIDATES => [Expect::unknown(&["x", "T"])],
]);

#[test]
fn a_test_module_file_is_planted_under_its_parent() {
    let tests = Read::file("src/tests.rs", "use super::*;\nimpl Default for R {}\n");
    let root = Read::file("src/lib.rs", "struct R;\n#[cfg(test)]\nmod tests;\n").with_test_modules(vec![tests]);
    assert_eq!(Read::implementations(&[root]), [Expect::written("R", Expect::std(&["std", "default", "Default"]))]);
}
