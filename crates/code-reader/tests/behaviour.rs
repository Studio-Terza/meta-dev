use code_reader::Reader;
use lint_core::{SourceFile, SourceText};

const DERIVED: &str = "#[derive(Clone,\n    Debug)]\nstruct A;\n#[derive(PartialEq)]\nenum B {\n    C,\n}\n#[derive(Copy)]\nunion D {\n    e: u8,\n}\n";
const WRITTEN: &str = "mod m {\n    impl Clone for A {}\n}\nfn f() {\n    impl A {}\n}\n";
const MIXED: &str = "#[derive(Debug)]\nstruct A;\nimpl Clone for A {}\n";

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
