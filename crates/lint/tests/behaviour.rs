use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, Disk, Files, Memory};
use lint::{Cli, Verdict};
use test_failure::Failed;
use timer_ports::Monotonic;

const GOLDENS: &str = "tests/goldens";
const GOLDEN_EXTENSION: &str = "txt";
const RECORD: &str = "RECORD_GOLDENS";
const NEWLINE: char = '\n';
const SPACE: char = ' ';
const OUT: &str = "--- out\n";
const ERR: &str = "--- err\n";
const MANIFEST: &str = "Cargo.toml";
const SOURCE: &str = "src/lib.rs";
const FREE: &str = "tests/.fixtures/current";
const FREE_SOURCE: &str = "tests/.fixtures/current/src/lib.rs";
const DEEP_MANIFEST: &str = "tests/.fixtures/deep/Cargo.toml";
const DEEP_HEAD: &str = "struct Deep;\nimpl Deep {\n    fn deep() -> Deep {\n        ";
const DEEP_CORE: &str = "Deep";
const DEEP_TAIL: &str = "\n    }\n}\n";
const OPEN: &str = "(";
const CLOSE: &str = ")";
const DEPTH: usize = 5_000;
const WORKSPACE: &str = "tests/.fixtures/workspace";
const UNWRITABLE: &str = "tests/goldens/unwritable.txt";
const UNPACKAGED: &str = "tests/goldens/unpackaged.txt";
const PACKAGED: &str = "tests/goldens/packaged.txt";
const DEEP: &str = "tests/goldens/deep.txt";
const REPORT: &str = "--report";
const VERDICTS: [Verdict; 3] = [Verdict::Clean, Verdict::Violations, Verdict::Refused];
const WORDS: [&str; 3] = ["Clean\n", "Violations\n", "Refused\n"];

const CASES: [(&str, &str, &[&str]); 29] = [
    ("bool_methods", "tests/.fixtures/bool_methods", &["tests/.fixtures/bool_methods"]),
    ("dead_code", "tests/.fixtures/dead_code", &["tests/.fixtures/dead_code"]),
    ("expression_flow", "tests/.fixtures/expression_flow", &["tests/.fixtures/expression_flow"]),
    ("manual_iteration", "tests/.fixtures/manual_iteration", &["tests/.fixtures/manual_iteration"]),
    ("named_lifetimes", "tests/.fixtures/named_lifetimes", &["tests/.fixtures/named_lifetimes"]),
    ("no_comments", "tests/.fixtures/no_comments", &["tests/.fixtures/no_comments"]),
    ("no_dyn", "tests/.fixtures/no_dyn", &["tests/.fixtures/no_dyn"]),
    ("no_free_fns", "tests/.fixtures/no_free_fns", &["tests/.fixtures/no_free_fns"]),
    ("no_test_only_api", "tests/.fixtures/no_test_only_api", &["tests/.fixtures/no_test_only_api"]),
    ("primitive_boundary", "tests/.fixtures/primitive_boundary", &["tests/.fixtures/primitive_boundary"]),
    ("process_starters", "tests/.fixtures/process_starters", &["tests/.fixtures/process_starters"]),
    ("public_api_tests", "tests/.fixtures/public_api_tests", &["tests/.fixtures/public_api_tests"]),
    ("public_surface", "tests/.fixtures/public_surface", &["tests/.fixtures/public_surface"]),
    ("test_shape", "tests/.fixtures/test_shape", &["tests/.fixtures/test_shape"]),
    ("type_shape", "tests/.fixtures/type_shape", &["tests/.fixtures/type_shape"]),
    ("vec_out_params", "tests/.fixtures/vec_out_params", &["tests/.fixtures/vec_out_params"]),
    ("workspace", WORKSPACE, &[WORKSPACE]),
    ("workspace_report", WORKSPACE, &[REPORT, WORKSPACE]),
    ("workspace_timings", WORKSPACE, &["--timings", WORKSPACE]),
    ("two_roots", WORKSPACE, &[REPORT, "tests/.fixtures/workspace/app", "tests/.fixtures/workspace/store-ports"]),
    ("flow", "tests/.fixtures/flow", &["tests/.fixtures/flow"]),
    ("starters", "tests/.fixtures/starters", &["tests/.fixtures/starters"]),
    ("surface", "tests/.fixtures/surface", &["tests/.fixtures/surface"]),
    ("test_code", "tests/.fixtures/test_code", &["tests/.fixtures/test_code"]),
    ("broken", "tests/.fixtures/broken", &["tests/.fixtures/broken"]),
    ("broken_report", "tests/.fixtures/broken", &[REPORT, "tests/.fixtures/broken"]),
    ("bad_manifest", "tests/.fixtures/bad_manifest", &["tests/.fixtures/bad_manifest"]),
    ("unreadable", "tests/.fixtures/bad_manifest", &["/nowhere", "tests/.fixtures/bad_manifest/notes"]),
    ("mounted", "tests/.fixtures/no_dyn", &["./tests/.fixtures/no_dyn"]),
];

#[test]
fn every_case_prints_its_golden_output() -> Result<(), Failed> {
    let drifted = CASES
        .iter()
        .map(|(golden, fixture, arguments)| {
            let files = Memory::default();
            Disk.mirror(Path::new(fixture), &files)?;
            let mut out = Vec::new();
            let mut err = Vec::new();
            let given = arguments.iter().map(OsString::from);
            let verdict = Cli::from_args(given).verdict(&files, Monotonic::default(), &mut out, &mut err);
            let letters: Vec<char> = String::from_utf8(out)?.chars().collect();
            let unnumbered: String = letters
                .chunk_by(|letter, _| *letter != NEWLINE)
                .flat_map(|line| line.iter().skip_while(|letter| **letter == SPACE || letter.is_ascii_digit()))
                .collect();
            let said = VERDICTS.iter().zip(WORDS).filter(|(known, _)| **known == verdict).map(|(_, word)| word);
            let shown: String = said.chain([OUT, &unnumbered, ERR, &String::from_utf8(err)?]).collect();
            let path = Path::new(GOLDENS).join(golden).with_extension(GOLDEN_EXTENSION);
            std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&path, Bytes::new(shown.clone().into_bytes())))?;
            let expected = Disk.read_text(&path)?;
            Ok((expected.as_ref() != shown).then_some(*golden))
        })
        .collect::<Result<Vec<Option<&str>>, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<&str>>(), Vec::<&str>::new());
    Ok(())
}

#[test]
fn no_argument_reads_the_current_directory() -> Result<(), Failed> {
    let files: Memory = [(PathBuf::from(SOURCE), Disk.read(Path::new(FREE_SOURCE))?)].into_iter().collect();
    let mut unpackaged = Vec::new();
    let refused = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut unpackaged);
    files.write(Path::new(MANIFEST), Disk.read(&Path::new(FREE).join(MANIFEST))?)?;
    let mut packaged = Vec::new();
    let violated = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut packaged);
    let expected = [Disk.read(Path::new(UNPACKAGED))?, Disk.read(Path::new(PACKAGED))?];
    assert_eq!(
        (refused, violated, [Bytes::new(unpackaged), Bytes::new(packaged)]),
        (Verdict::Refused, Verdict::Violations, expected)
    );
    Ok(())
}

#[test]
fn verdicts_become_exit_codes() {
    let exits = [Verdict::Clean, Verdict::Violations, Verdict::Refused].map(ExitCode::from);
    assert_eq!(exits, [ExitCode::SUCCESS, ExitCode::FAILURE, ExitCode::FAILURE]);
}

#[test]
fn deeply_nested_code_is_read_whole() -> Result<(), Failed> {
    let nested = [DEEP_HEAD, &OPEN.repeat(DEPTH), DEEP_CORE, &CLOSE.repeat(DEPTH), DEEP_TAIL].concat();
    let files: Memory = [
        (PathBuf::from(MANIFEST), Disk.read(Path::new(DEEP_MANIFEST))?),
        (PathBuf::from(SOURCE), Bytes::new(nested.into_bytes())),
    ]
    .into_iter()
    .collect();
    let mut err = Vec::new();
    let verdict = Cli::from_args([]).verdict(&files, Monotonic::default(), &mut Vec::new(), &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Clean, Disk.read(Path::new(DEEP))?));
    Ok(())
}

#[test]
fn output_that_cannot_be_written_refuses_the_run() -> Result<(), Failed> {
    let files = Memory::default();
    Disk.mirror(Path::new(WORKSPACE), &files)?;
    let mut err = Vec::new();
    let mut nowhere: &mut [u8] = &mut [];
    let arguments = [REPORT, WORKSPACE].map(OsString::from);
    let verdict = Cli::from_args(arguments).verdict(&files, Monotonic::default(), &mut nowhere, &mut err);
    assert_eq!((verdict, Bytes::new(err)), (Verdict::Refused, Disk.read(Path::new(UNWRITABLE))?));
    Ok(())
}
