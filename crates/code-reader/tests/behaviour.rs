use code_model::{Implementation, Origin};
use code_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::Name;

const DERIVED: &str = "#[derive(Clone,\n    Debug)]\nstruct A;\n#[derive(PartialEq)]\nenum B {\n    C,\n}\n#[derive(Copy)]\nunion D {\n    e: u8,\n}\n";
const WRITTEN: &str = "mod m {\n    impl Clone for A {}\n}\nfn f() {\n    impl A {}\n}\n";
const MIXED: &str = "#[derive(Debug)]\nstruct A;\nimpl Clone for A {}\n";

const RENAMED: &str = "use std::{clone::Clone as Dup};\nimpl Dup for R {}\nimpl a::Dup for R {}\n";
const MACROS: &str = "macro_rules! make {\n    ($t:ty, $u:ty) => {\n        impl<T> Clone for Wrapper<T> {}\n        impl PartialEq<$u> for $t {}\n        impl $t {\n            fn each() { for x in y {} }\n        }\n        mod inner {\n            impl Copy for Inner {}\n        }\n    };\n}\nfn f() {\n    other! {\n        impl Eq for Skipped {}\n    }\n}\n";

struct Expect;

impl Expect {
    fn written(subject: &str, path: &[&str]) -> Implementation {
        Implementation::new(
            Name::from(subject),
            Some(path.iter().map(|segment| Name::from(*segment)).collect()),
            Origin::Written,
        )
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
        [Expect::written("R", &["Clone"]), Expect::written("R", &["a", "Clone"]),]
    );
}

#[test]
fn only_macro_rules_bodies_yield_trait_impls_in_macros() {
    assert_eq!(
        Read::implementations(&[Read::file("a.rs", MACROS)]),
        [
            Expect::written("Wrapper", &["Clone"]),
            Expect::written("", &["PartialEq"]),
            Expect::written("Inner", &["Copy"]),
        ]
    );
}
