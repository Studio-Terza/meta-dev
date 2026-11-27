use std::ffi::OsString;
use std::path::PathBuf;

use file_ports::{Bytes, Memory};
use lint::{Cli, Verdict};
use proptest::collection::vec;
use proptest::prelude::{Strategy, any, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::select;
use timer_ports::Monotonic;

const MANIFEST: &str = "Cargo.toml";
const MANIFEST_TEXT: &str = "[package]\nname = \"fuzzed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const PLACES: [&str; 5] = ["src/lib.rs", "src/main.rs", "tests/fuzzed.rs", "build.rs", "Cargo.toml"];
const MAX_BYTES: usize = 512;
const MAX_PARTS: usize = 64;
const MAX_NESTING: usize = 140;
const MAX_CHAIN: usize = 30_000;
const OPEN: [&str; 4] = ["(", "{ ", "[", "-"];
const CLOSE: [&str; 4] = [")", " }", "]", ""];
const CHAINED: &str = "x";
const LINKS: [&str; 6] = [".m()", " + 1", "?", " as u8", ".0", " && x"];
const FRAGMENTS: [&str; 48] = [
    "fn ",
    "pub ",
    "pub(crate) ",
    "struct ",
    "enum ",
    "impl ",
    "trait ",
    "mod ",
    "use ",
    "const ",
    "type ",
    "let ",
    "mut ",
    "for ",
    "while ",
    "loop ",
    "if ",
    "else ",
    "match ",
    "dyn ",
    "x",
    "Foo",
    "T",
    "'a",
    "u8",
    "String",
    "&str",
    "bool",
    "-> bool",
    "(",
    ")",
    "{",
    "}",
    "<",
    ">",
    ";",
    ":",
    "::",
    ",",
    "=",
    "#[test]",
    "#[cfg(test)]",
    "// note",
    "\"text\"",
    "1",
    "proptest! {",
    "macro_rules! m { ($x:expr) => { $x }; }",
    "std::process::Command::new(\"ls\")",
];

proptest! {
    #[test]
    fn any_input_ends_in_a_verdict_that_matches_what_was_said(
        (place, bytes) in prop_oneof![
            (select(PLACES.to_vec()), vec(any::<u8>(), 0..MAX_BYTES)),
            (select(PLACES.to_vec()), vec(select(FRAGMENTS.to_vec()), 0..MAX_PARTS)
                .prop_map(|parts| parts.concat().into_bytes())),
            (select(PLACES.to_vec()), (0..OPEN.len(), 1..MAX_NESTING)
                .prop_map(|(shell, depth)| {
                    let open = OPEN.get(shell).copied().unwrap_or_default().repeat(depth);
                    let close = CLOSE.get(shell).copied().unwrap_or_default().repeat(depth);
                    [open, CHAINED.to_owned(), close].concat().into_bytes()
                })),
            (select(PLACES.to_vec()), vec(select(LINKS.to_vec()), 0..MAX_CHAIN)
                .prop_map(|links| [CHAINED].into_iter().chain(links).collect::<String>().into_bytes())),
        ]
    ) {
        let files: Memory = [
            (PathBuf::from(MANIFEST), Bytes::new(MANIFEST_TEXT.as_bytes().to_vec())),
            (PathBuf::from(place), Bytes::new(bytes)),
        ]
        .into_iter()
        .collect();
        let mut err = Vec::new();
        let verdict = Cli::from_args(Vec::<OsString>::new()).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut err);
        prop_assert_eq!(verdict == Verdict::Clean, err.is_empty());
    }
}
