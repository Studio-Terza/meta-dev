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

#[derive(Debug)]
struct Case {
    golden: &'static str,
    fixture: &'static str,
    arguments: &'static [&'static str],
}

#[derive(Debug)]
struct Said {
    verdict: Verdict,
    word: &'static str,
}

const SAID: [Said; 3] = [
    Said { verdict: Verdict::Clean, word: "Clean\n" },
    Said { verdict: Verdict::Violations, word: "Violations\n" },
    Said { verdict: Verdict::Refused, word: "Refused\n" },
];

const CASES: [Case; 29] = [
    Case {
        golden: "bool_methods",
        fixture: "tests/.fixtures/bool_methods",
        arguments: &["tests/.fixtures/bool_methods"],
    },
    Case { golden: "dead_code", fixture: "tests/.fixtures/dead_code", arguments: &["tests/.fixtures/dead_code"] },
    Case {
        golden: "expression_flow",
        fixture: "tests/.fixtures/expression_flow",
        arguments: &["tests/.fixtures/expression_flow"],
    },
    Case {
        golden: "manual_iteration",
        fixture: "tests/.fixtures/manual_iteration",
        arguments: &["tests/.fixtures/manual_iteration"],
    },
    Case {
        golden: "named_lifetimes",
        fixture: "tests/.fixtures/named_lifetimes",
        arguments: &["tests/.fixtures/named_lifetimes"],
    },
    Case { golden: "no_comments", fixture: "tests/.fixtures/no_comments", arguments: &["tests/.fixtures/no_comments"] },
    Case { golden: "no_dyn", fixture: "tests/.fixtures/no_dyn", arguments: &["tests/.fixtures/no_dyn"] },
    Case { golden: "no_free_fns", fixture: "tests/.fixtures/no_free_fns", arguments: &["tests/.fixtures/no_free_fns"] },
    Case {
        golden: "no_test_only_api",
        fixture: "tests/.fixtures/no_test_only_api",
        arguments: &["tests/.fixtures/no_test_only_api"],
    },
    Case {
        golden: "primitive_boundary",
        fixture: "tests/.fixtures/primitive_boundary",
        arguments: &["tests/.fixtures/primitive_boundary"],
    },
    Case {
        golden: "process_starters",
        fixture: "tests/.fixtures/process_starters",
        arguments: &["tests/.fixtures/process_starters"],
    },
    Case {
        golden: "public_api_tests",
        fixture: "tests/.fixtures/public_api_tests",
        arguments: &["tests/.fixtures/public_api_tests"],
    },
    Case {
        golden: "public_surface",
        fixture: "tests/.fixtures/public_surface",
        arguments: &["tests/.fixtures/public_surface"],
    },
    Case { golden: "test_shape", fixture: "tests/.fixtures/test_shape", arguments: &["tests/.fixtures/test_shape"] },
    Case { golden: "type_shape", fixture: "tests/.fixtures/type_shape", arguments: &["tests/.fixtures/type_shape"] },
    Case {
        golden: "vec_out_params",
        fixture: "tests/.fixtures/vec_out_params",
        arguments: &["tests/.fixtures/vec_out_params"],
    },
    Case { golden: "workspace", fixture: WORKSPACE, arguments: &[WORKSPACE] },
    Case { golden: "workspace_report", fixture: WORKSPACE, arguments: &[REPORT, WORKSPACE] },
    Case { golden: "workspace_timings", fixture: WORKSPACE, arguments: &["--timings", WORKSPACE] },
    Case {
        golden: "two_roots",
        fixture: WORKSPACE,
        arguments: &[REPORT, "tests/.fixtures/workspace/app", "tests/.fixtures/workspace/store-ports"],
    },
    Case { golden: "flow", fixture: "tests/.fixtures/flow", arguments: &["tests/.fixtures/flow"] },
    Case { golden: "starters", fixture: "tests/.fixtures/starters", arguments: &["tests/.fixtures/starters"] },
    Case { golden: "surface", fixture: "tests/.fixtures/surface", arguments: &["tests/.fixtures/surface"] },
    Case { golden: "test_code", fixture: "tests/.fixtures/test_code", arguments: &["tests/.fixtures/test_code"] },
    Case { golden: "broken", fixture: "tests/.fixtures/broken", arguments: &["tests/.fixtures/broken"] },
    Case { golden: "broken_report", fixture: "tests/.fixtures/broken", arguments: &[REPORT, "tests/.fixtures/broken"] },
    Case {
        golden: "bad_manifest",
        fixture: "tests/.fixtures/bad_manifest",
        arguments: &["tests/.fixtures/bad_manifest"],
    },
    Case {
        golden: "unreadable",
        fixture: "tests/.fixtures/bad_manifest",
        arguments: &["/nowhere", "tests/.fixtures/bad_manifest/notes"],
    },
    Case { golden: "mounted", fixture: "tests/.fixtures/no_dyn", arguments: &["./tests/.fixtures/no_dyn"] },
];

#[test]
fn every_case_prints_its_golden_output() -> Result<(), Failed> {
    let drifted = CASES
        .iter()
        .map(|case| {
            let files = Memory::default();
            Disk.mirror(Path::new(case.fixture), &files)?;
            let mut out = Vec::new();
            let mut err = Vec::new();
            let given = case.arguments.iter().map(OsString::from);
            let verdict = Cli::from_args(given).verdict(&files, Monotonic::default(), &mut out, &mut err);
            let letters: Vec<char> = String::from_utf8(out)?.chars().collect();
            let unnumbered: String = letters
                .chunk_by(|letter, _| *letter != NEWLINE)
                .flat_map(|line| line.iter().skip_while(|letter| **letter == SPACE || letter.is_ascii_digit()))
                .collect();
            let said = SAID.iter().filter(|said| said.verdict == verdict).map(|said| said.word);
            let shown: String = said.chain([OUT, &unnumbered, ERR, &String::from_utf8(err)?]).collect();
            let golden = Path::new(GOLDENS).join(case.golden).with_extension(GOLDEN_EXTENSION);
            std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&golden, Bytes::new(shown.clone().into_bytes())))?;
            let expected = Disk.read_text(&golden)?;
            Ok((expected.as_ref() != shown).then_some(case.golden))
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
