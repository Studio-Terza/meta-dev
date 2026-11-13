use std::path::PathBuf;

use code_reader::Reader;
use file_ports::{Bytes, Memory};
use lint_core::{ModelLint, SourceFile, SourceText};
use lint_no_test_io::NoTestIo;
use lint_placement::Placement;

use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};

const READ: &str = "fn f() {\n    std::fs::read_to_string(\"a\");\n}\n";
const PARENT: &str = "fn f() {\n    std::fs::read_to_string(\"a\");\n}\nmod b;\n";
const IMPORTED: &str = "use std::fs;\nfn f() {\n    fs::write(\"a\", \"b\");\n}\n";
const TYPE: &str = "use std::fs::File;\nfn f() {\n    File::open(\"a\");\n}\n";
const GLOB: &str = "use std::env::*;\nfn f() {\n    var(\"A\");\n}\n";
const RENAMED: &str = "use std::net::TcpStream as Socket;\nfn f() {\n    Socket::connect(\"a:1\");\n}\n";
const TEMPORARY: &str = "fn f() {\n    tempfile::tempdir();\n}\n";
const STARTED: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const EXITED: &str = "fn f() {\n    std::process::exit(1);\n}\n";
const LINKED: &str = "fn f() {\n    std::os::unix::fs::symlink(\"a\", \"b\");\n}\n";
const IN_MACRO: &str = "fn f() {\n    assert!(std::fs::read(\"a\").is_ok());\n}\n";
const EXIT_CODE: &str = "use std::process::ExitCode;\nfn f() -> ExitCode {\n    ExitCode::SUCCESS\n}\n";
const COMPILED: &str = "use std::env;\nconst A: &str = concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/a\");\n";
const TYPED: &str = "fn f(_file: std::fs::File) -> std::io::Result<()> {\n    Ok(())\n}\n";
const OTHER_MODULE: &str = "fn f() {\n    std::path::Path::new(\"a\");\n}\n";
const LOCAL_FS: &str = "mod fs {\n    pub fn read() {}\n}\nfn f() {\n    fs::read();\n}\n";
const INLINE_PORTS: &str = "mod ports {\n    pub fn f() {\n        std::fs::read(\"a\");\n    }\n}\n";
const DISK: &str = "use file_ports::{Disk, Files};\nfn f() {\n    Disk.read(std::path::Path::new(\"a\"));\n}\n";
const PROCESS: &str = "fn f() {\n    console_ports::Process.variable(\"A\".as_ref());\n}\n";
const GOLDEN_PORTS: &str = "use golden::{Disk, Process};\nfn f() {\n    Golden::verify(&Process, &Disk);\n}\n";
const PRINTED: &str = "use std::io::{self, Write};\nfn f() {\n    writeln!(io::stdout(), \"a\");\n}\n";
const FAKES: &str =
    "use file_ports::{Files, Memory};\nfn f() {\n    Memory::default().read(std::path::Path::new(\"a\"));\n}\n";

const CASES: [Case; 28] = [
    Case::new(
        Title::new("qualified read"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(READ))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("imported module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(IMPORTED))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("imported type"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TYPE))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("glob"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GLOB))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("renamed socket"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(RENAMED))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("temporary directory"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TEMPORARY))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("started process"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(STARTED))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("exit"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(EXITED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("platform file system"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LINKED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("inside an assertion"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("module of a test target"),
        &[
            Source::new(Located::new("tests/behaviour/main.rs"), Text::new("mod a;\n")),
            Source::new(Located::new("tests/behaviour/a.rs"), Text::new(READ)),
        ],
        Flagged::new(&["tests/behaviour/a.rs:2"]),
    ),
    Case::new(
        Title::new("exit code is a value"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(EXIT_CODE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("env macro is read at compile time"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(COMPILED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("types are not effects"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TYPED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("another std module"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(OTHER_MODULE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a local module named fs"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LOCAL_FS))],
        Flagged::new(&[]),
    ),
    Case::new(Title::new("shipped code"), &[Source::new(Located::new("src/a.rs"), Text::new(READ))], Flagged::new(&[])),
    Case::new(Title::new("example"), &[Source::new(Located::new("examples/a.rs"), Text::new(READ))], Flagged::new(&[])),
    Case::new(
        Title::new("the real file port"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(DISK))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("the real console port"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PROCESS))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("the real ports through golden"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GOLDEN_PORTS))],
        Flagged::new(&["tests/a.rs:3", "tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("standard output"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PRINTED))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("the in-memory fakes"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(FAKES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("test target named ports"),
        &[
            Source::new(Located::new("tests/ports.rs"), Text::new(PARENT)),
            Source::new(Located::new("tests/ports/b.rs"), Text::new(READ)),
        ],
        Flagged::new(&["tests/ports/b.rs:2", "tests/ports.rs:2"]),
    ),
    Case::new(
        Title::new("test target whose name only contains ports"),
        &[
            Source::new(Located::new("tests/reports.rs"), Text::new(READ)),
            Source::new(Located::new("tests/disk_ports.rs"), Text::new(READ)),
        ],
        Flagged::new(&["tests/disk_ports.rs:2", "tests/reports.rs:2"]),
    ),
    Case::new(
        Title::new("module named ports"),
        &[
            Source::new(Located::new("tests/behaviour/main.rs"), Text::new("mod ports;\n")),
            Source::new(Located::new("tests/behaviour/ports.rs"), Text::new(PARENT)),
            Source::new(Located::new("tests/behaviour/ports/b.rs"), Text::new(READ)),
        ],
        Flagged::new(&["tests/behaviour/ports/b.rs:2", "tests/behaviour/ports.rs:2"]),
    ),
    Case::new(
        Title::new("inline module named ports"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(INLINE_PORTS))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("a crate whose name only contains ports"),
        &[Source::new(Located::new("crates/reports/tests/a.rs"), Text::new(READ))],
        Flagged::new(&["crates/reports/tests/a.rs:2"]),
    ),
];

#[test]
fn effects_and_real_ports_in_test_code_are_flagged_outside_ports_crates() {
    assert_eq!(Sources::mismatches::<Queried<NoTestIo>>(&CASES), Lines::default());
}

#[test]
fn tests_of_a_crate_whose_name_ends_in_ports_check_its_real_fillers() {
    let flagged: Vec<usize> = ["disk-ports", "reports"]
        .iter()
        .map(|name| {
            let manifest = format!("[package]\nname = \"{name}\"\n");
            let files: Memory =
                [(PathBuf::from("/r/Cargo.toml"), Bytes::from(manifest.as_str()))].into_iter().collect();
            let path = PathBuf::from("/r/tests/a.rs");
            let placement = Placement::locate(&files, &path);
            SourceFile::parse_at(path.clone(), path, placement, SourceText::new(DISK.to_owned()))
                .map_or(usize::MAX, |file| NoTestIo::report(&Reader::read_files(&[file])).violations().len())
        })
        .collect();
    assert_eq!(flagged, [0, 1]);
}
