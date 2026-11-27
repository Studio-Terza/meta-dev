use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Memory};
use lint::{Cli, Verdict};
use proptest::collection::vec;
use proptest::prelude::{Strategy, any, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::select;
use proptest::test_runner::TestCaseError;
use timer_ports::Monotonic;

proptest! {
    #[test]
    fn any_input_ends_in_a_verdict_that_matches_what_was_said(
        place in select(vec!["src/lib.rs", "src/main.rs", "tests/fuzzed.rs", "build.rs", "Cargo.toml"]),
        bytes in prop_oneof![
            vec(any::<u8>(), 0..512),
            vec(
                select(vec![
                    "fn ", "pub ", "pub(crate) ", "struct ", "enum ", "impl ", "trait ", "mod ", "use ", "const ",
                    "type ", "let ", "mut ", "for ", "while ", "loop ", "if ", "else ", "match ", "dyn ", "x", "Foo",
                    "T", "'a", "u8", "String", "&str", "bool", "-> bool", "(", ")", "{", "}", "<", ">", ";", ":",
                    "::", ",", "=", "#[test]", "#[cfg(test)]", "// note", "\"text\"", "1", "proptest! {",
                    "macro_rules! m { ($x:expr) => { $x }; }", "std::process::Command::new(\"ls\")",
                ]),
                0..64,
            )
            .prop_map(|parts| parts.concat().into_bytes()),
            (select(vec![("(", ")"), ("{ ", " }"), ("[", "]"), ("-", "")]), 1..140_usize)
                .prop_map(|((open, close), depth)| [open.repeat(depth), close.repeat(depth)].join("x").into_bytes()),
            vec(select(vec![".m()", " + 1", "?", " as u8", ".0", " && x"]), 0..30_000)
                .prop_map(|links| ["x".to_owned(), links.concat()].concat().into_bytes()),
        ]
    ) {
        let source = Path::new(file!());
        let table = source.with_file_name(source.file_stem().unwrap_or_default());
        let found = std::env::current_dir()?
            .ancestors()
            .map(|directory| directory.join(&table))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        let (manifest_at, manifest_from): (PathBuf, PathBuf) = serde_json::from_slice(&std::fs::read(found)?)
            .map_err(|failure| TestCaseError::fail(failure.to_string()))?;
        let files: Memory = [
            (manifest_at, Bytes::new(std::fs::read(manifest_from)?)),
            (PathBuf::from(place), Bytes::new(bytes)),
        ]
        .into_iter()
        .collect();
        let mut err = Vec::new();
        let verdict = Cli::from_args(Vec::<OsString>::new()).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut err);
        prop_assert_eq!(verdict == Verdict::Clean, err.is_empty());
    }
}
