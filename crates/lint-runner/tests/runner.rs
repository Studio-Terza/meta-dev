use lint_reader::SourceFiles;
use lint_runner::{Cli, Ports, Verdict};
use test_failure::Failed;
use timer_ports::Monotonic;

use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use cargo_ports::Recorded;
use file_ports::{Bytes, FileFailure, Files, Memory};

use crate::fakes::{FakeCounted, FakeLinked, FakeUnlistable, FakeUnsized};
use crate::ignored::Ignored;
use invariants::Ignoring;

const DIRTY: &str = "// note\nfn render_page() {}\n";
const CLEAN: &str = "struct A;\n";
const LONG_CHAIN_LINKS: usize = 20_000;
const DEEP_NESTING: usize = 5_000;
const SHALLOW_NESTING: usize = 100;
const MIDDLING_NESTING: usize = 200;
const MANIFEST: &str = "[package]\nname = \"r\"\n";
const MOST_READERS: usize = 4;
const NO_DEPENDENCIES: &str = "{\"workspace_root\": \"/\", \"packages\": []}";

#[test]
fn every_lint_reports_over_the_same_files() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let run = root.reported();
    assert_eq!(run.violations(), 2);
    assert_eq!(run.headlines(), 2);
    assert!(run.err.contains("src/lib.rs:1"));
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn a_tree_with_violations_is_not_clean() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    root.write("src/clean.rs", CLEAN)?;
    let run = root.reported();
    assert_eq!(run.headlines(), 2);
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn a_clean_tree_is_clean_and_build_directories_are_not_read() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    root.write("target/out.rs", DIRTY)?;
    assert_eq!(root.reported(), Reported::clean());
    Ok(())
}

#[test]
fn the_report_flag_prints_the_test_budget_and_blocks_nothing() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let surveyed = root.flagged(&["--report"]);
    assert_eq!(surveyed.out.matches("test budget, report only:\n").count(), 1, "{}", surveyed.out);
    assert_eq!(surveyed.exit, ExitCode::SUCCESS);
    assert_eq!(surveyed.err, "");
    Ok(())
}

#[test]
fn the_report_flag_fails_on_an_unreadable_root() {
    let run = Trees::flagged(&Memory::default(), Path::new("/no/such/root"), &["--report"]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert_ne!(run.err, "");
}

#[test]
fn the_gate_refuses_a_workspace_cargo_cannot_describe() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let gate = Cli::from_args([OsString::from(&root.root)].into_iter()).report_to(
        Ports::new(
            &root.files,
            [(PathBuf::from("/"), Bytes::from("not json"))].into_iter().collect::<Recorded>(),
            Monotonic::default(),
        ),
        &mut out,
        &mut err,
    );
    let (mut report_out, mut report_err) = (Vec::new(), Vec::new());
    let report = Cli::from_args([OsString::from("--report"), OsString::from(&root.root)].into_iter()).report_to(
        Ports::new(
            &root.files,
            [(PathBuf::from("/"), Bytes::from("not json"))].into_iter().collect::<Recorded>(),
            Monotonic::default(),
        ),
        &mut report_out,
        &mut report_err,
    );
    assert_eq!(gate, ExitCode::FAILURE);
    assert_ne!(err, Vec::<u8>::new());
    assert_eq!(out, Vec::<u8>::new());
    assert_eq!(report, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn the_verdict_names_what_the_run_decided() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "pub struct A;\n")?;
    let clean = root.verdict(&[]);
    root.write("src/lib.rs", DIRTY)?;
    let mut err = Vec::new();
    let unwritten = [&["--timings"][..], &["--report"]].map(|flags| {
        Cli::from_args(flags.iter().map(OsString::from).chain([OsString::from(&root.root)])).verdict(
            Ports::new(
                &root.files,
                [(PathBuf::from("/"), Bytes::from(NO_DEPENDENCIES))].into_iter().collect::<Recorded>(),
                Monotonic::default(),
            ),
            &mut <&mut [u8]>::default(),
            &mut err,
        )
    });
    let unreadable = Trees::verdict(&Memory::default(), Path::new("/no/such/root"), &[]);
    assert_eq!([clean, root.verdict(&[]), unreadable], [Verdict::Clean, Verdict::Violations, Verdict::Refused]);
    assert_eq!(unwritten, [Verdict::Refused; 2]);
    assert_eq!(String::from_utf8(err)?.matches("cannot write the output").count(), 2);
    Ok(())
}

#[test]
fn a_cargo_metadata_failure_refuses_the_run_once_rather_than_flagging_each_derive() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#[derive(serde::Deserialize)]\npub struct A {\n    a: Title,\n}\n")?;
    let mut err = Vec::new();
    let arguments = [OsString::from(&root.root)].into_iter();
    let verdict = Cli::from_args(arguments).verdict(
        Ports::new(&root.files, Recorded::default(), Monotonic::default()),
        &mut Vec::new(),
        &mut err,
    );
    assert_eq!(
        (verdict, String::from_utf8(err)?.parse::<text_lines::Lines>().into_iter().flatten().count()),
        (Verdict::Refused, 1)
    );
    Ok(())
}

#[test]
fn a_violation_fails_the_run() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn the_gate_lists_the_tree_once_and_reads_each_source_once() -> Result<(), Failed> {
    let scratch = Scratch::package()?;
    scratch.write("src/lib.rs", CLEAN)?;
    let reading = FakeCounted::new(&scratch.files, scratch.root.clone());
    Trees::flagged(&reading, &scratch.root, &["--report"]);
    let gating = FakeCounted::new(&scratch.files, scratch.root.clone());
    let gate = Trees::flagged(&gating, &scratch.root, &[]);
    assert_eq!(gate.exit, ExitCode::SUCCESS);
    assert_eq!(gating.listings(), reading.listings());
    assert_eq!([gating.sources(), reading.sources()], [1, 1]);
    Ok(())
}

#[test]
fn the_timings_flag_lists_every_lint_and_phase_once() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    let timed = Trees::timed(&root.files, &root.root, &["--timings"])?;
    let untimed = Trees::timed(&root.files, &root.root, &[])?;
    let workers =
        thread::available_parallelism().ignoring(Ignored::Unknown).map_or(1, NonZeroUsize::get).min(MOST_READERS);
    let rows = [
        "lint timings in milliseconds, took then started:\n",
        "  phase total\n",
        "  phase stage: list the trees and their packages\n",
        "  phase locate: describe the workspaces through cargo metadata\n",
        "  phase contexts: join every file's context-lint parts\n",
    ];
    assert_eq!(rows.map(|row| timed.matches(row).count()), [1; 5], "{timed}");
    let waited = "  wait judged lints on worker ";
    assert_eq!(timed.matches(waited).count(), workers, "{timed}");
    assert_eq!(timed.matches("  phase read: read and parse the sources on worker ").count(), workers, "{timed}");
    let lowered = "  phase lower: lower and file-lint its share of files on worker ";
    assert_eq!(timed.matches(lowered).count(), workers, "{timed}");
    assert_eq!(untimed.matches("lint timings").count(), 0, "{untimed}");
    assert_eq!(timed.matches("      0  phase total\n").count(), 1, "{timed}");
    Ok(())
}

#[test]
fn the_timings_flag_counts_only_the_subtrees_that_hold_the_kinds_a_contributor_watches() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "struct A {\n    b: u8,\n}\nimpl A {}\n")?;
    let timed = root.flagged(&["--timings"]).out;
    assert_eq!(timed.matches("\n      2  lint_bool_methods::returns::Returns<'_>\n").count(), 1, "{timed}");
    Ok(())
}

struct Trees;

impl Trees {
    fn flagged(files: &(impl Files + Sync), root: &Path, flags: &[&str]) -> Printed {
        Trees::described(files, root, flags, NO_DEPENDENCIES)
    }

    fn described(files: &(impl Files + Sync), root: &Path, flags: &[&str], metadata: &str) -> Printed {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let arguments = flags.iter().map(OsString::from).chain([OsString::from(root)]);
        let exit = Cli::from_args(arguments).report_to(
            Ports::new(
                files,
                [(PathBuf::from("/"), Bytes::from(metadata))].into_iter().collect::<Recorded>(),
                Monotonic::default(),
            ),
            &mut out,
            &mut err,
        );
        Printed {
            exit,
            out: String::from_utf8(out).ignoring(Ignored::NotUtf8).unwrap_or_default(),
            err: String::from_utf8(err).ignoring(Ignored::NotUtf8).unwrap_or_default(),
        }
    }

    fn timed(files: &(impl Files + Sync), root: &Path, flags: &[&str]) -> Result<String, Failed> {
        let mut out = Vec::new();
        let arguments = flags.iter().map(OsString::from).chain([OsString::from(root)]);
        let exit = Cli::from_args(arguments).report_to(
            Ports::new(
                files,
                [(PathBuf::from("/"), Bytes::from(NO_DEPENDENCIES))].into_iter().collect::<Recorded>(),
                Monotonic::default(),
            ),
            &mut out,
            &mut Vec::new(),
        );
        assert_eq!(exit, ExitCode::SUCCESS);
        Ok(String::from_utf8(out)?)
    }

    fn verdict(files: &(impl Files + Sync), root: &Path, flags: &[&str]) -> Verdict {
        let arguments = flags.iter().map(OsString::from).chain([OsString::from(root)]);
        Cli::from_args(arguments).verdict(
            Ports::new(
                files,
                [(PathBuf::from("/"), Bytes::from(NO_DEPENDENCIES))].into_iter().collect::<Recorded>(),
                Monotonic::default(),
            ),
            &mut Vec::new(),
            &mut Vec::new(),
        )
    }

    fn reported_through(files: &(impl Files + Sync), roots: &[PathBuf]) -> Reported {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let exit = Cli::from_args(roots.iter().map(OsString::from)).report_to(
            Ports::new(
                files,
                [(PathBuf::from("/"), Bytes::from(NO_DEPENDENCIES))].into_iter().collect::<Recorded>(),
                Monotonic::default(),
            ),
            &mut out,
            &mut err,
        );
        Reported { exit, err: format!("\n{}", String::from_utf8(err).ignoring(Ignored::NotUtf8).unwrap_or_default()) }
    }
}

struct Scratch {
    files: Memory,
    root: PathBuf,
}

impl Scratch {
    const ROOT: &'static str = "/scratch";

    fn temporary() -> Result<Scratch, FileFailure> {
        let files = Memory::default();
        files.create_directories(Path::new(Self::ROOT))?;
        Ok(Scratch { files, root: PathBuf::from(Self::ROOT) })
    }

    fn package() -> Result<Scratch, FileFailure> {
        let scratch = Self::temporary()?;
        scratch.write("Cargo.toml", MANIFEST)?;
        Ok(scratch)
    }

    fn at(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    fn write(&self, path: &str, text: &str) -> Result<PathBuf, FileFailure> {
        let file = self.at(path);
        file.parent().map_or(Ok(()), |parent| self.files.create_directories(parent))?;
        self.files.write(&file, Bytes::from(text))?;
        Ok(file)
    }

    fn write_below(&self, directory: &str, path: &str, text: &str) -> Result<PathBuf, FileFailure> {
        self.write(&format!("{directory}/{path}"), text)
    }

    fn directory(&self, path: &str) -> Result<PathBuf, FileFailure> {
        let directory = self.at(path);
        self.files.create_directories(&directory)?;
        Ok(directory)
    }

    fn reported(&self) -> Reported {
        Trees::reported_through(&self.files, std::slice::from_ref(&self.root))
    }

    fn judged(sources: &&[[&str; 2]]) -> Option<usize> {
        let root = Scratch::package().ignoring(Ignored::Unwritten)?;
        let written = sources.iter().try_for_each(|[path, text]| root.write(path, text).map(|_| ()));
        written.ignoring(Ignored::Unwritten).map(|()| root.reported().violations())
    }

    fn reported_at(&self, below: &str) -> Reported {
        Trees::reported_through(&self.files, &[self.at(below)])
    }

    fn flagged(&self, flags: &[&str]) -> Printed {
        Trees::flagged(&self.files, &self.root, flags)
    }

    fn verdict(&self, flags: &[&str]) -> Verdict {
        Trees::verdict(&self.files, &self.root, flags)
    }

    fn read_files(&self) -> Result<Vec<String>, Failed> {
        let readout = SourceFiles::read(&self.files, &self.root)?;
        let shown = |file: &lint_core::SourceFile| {
            let path: &Path = file.head().as_ref();
            path.strip_prefix(&self.root).ignoring(Ignored::Outside).unwrap_or(path).display().to_string()
        };
        Ok(readout
            .files()
            .iter()
            .flat_map(|file| {
                let tests = file.test_modules().iter().map(|test| format!("test {}", shown(test)));
                [shown(file)].into_iter().chain(tests)
            })
            .collect())
    }
}

#[test]
fn a_module_file_two_packages_both_reach_is_read_once() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n")?;
    root.write("crates/a/Cargo.toml", "[package]\nname = \"a\"\n")?;
    root.write("crates/a/src/lib.rs", "#[path = \"../../shared.rs\"]\nmod shared;\n")?;
    root.write("crates/b/Cargo.toml", "[package]\nname = \"b\"\n")?;
    root.write("crates/b/src/lib.rs", "#[path = \"../../shared.rs\"]\nmod shared;\n")?;
    root.write("crates/shared.rs", CLEAN)?;
    assert_eq!(root.read_files()?, ["crates/a/src/lib.rs", "crates/b/src/lib.rs", "crates/shared.rs"]);
    Ok(())
}

#[test]
fn a_file_two_test_modules_name_is_read_once() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#[cfg(test)]\nmod t;\n#[cfg(test)]\n#[path = \"t.rs\"]\nmod again;\n")?;
    root.write("src/t.rs", CLEAN)?;
    assert_eq!(root.read_files()?, ["src/lib.rs", "test src/t.rs"]);
    Ok(())
}

#[test]
fn reading_a_module_path_cycle_finishes() -> Result<(), Failed> {
    let root = Scratch::package()?;
    PATH_CYCLE.iter().try_for_each(|[path, text]| root.write(path, text).map(|_| ()))?;
    let (sending, receiving) = mpsc::channel();
    thread::spawn(move || sending.send(root.read_files().map_err(|failure| format!("{failure:?}"))));
    let read = receiving.recv_timeout(Duration::from_secs(10))?;
    assert_eq!(read?, ["src/a.rs", "src/b.rs", "src/lib.rs"]);
    Ok(())
}

#[test]
fn a_panic_while_reading_reaches_the_caller_rather_than_stranding_the_helpers() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod a;\n")?;
    root.write("src/a.rs", CLEAN)?;
    let (sending, receiving) = mpsc::channel();
    thread::spawn(move || {
        let sizing = FakeUnsized::new(&root.files);
        let ran = panic::catch_unwind(AssertUnwindSafe(|| Trees::flagged(&sizing, &root.root, &[]).exit));
        sending.send(Verdicts::from(ran))
    });
    assert_eq!(receiving.recv_timeout(Duration::from_secs(10))?, Verdicts::Panicked);
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum Verdicts {
    Panicked,
    Returned,
}

impl From<thread::Result<ExitCode>> for Verdicts {
    fn from(ran: thread::Result<ExitCode>) -> Verdicts {
        match ran {
            Ok(_) => Verdicts::Returned,
            Err(_) => Verdicts::Panicked,
        }
    }
}

struct Printed {
    exit: ExitCode,
    out: String,
    err: String,
}

#[derive(Debug, PartialEq)]
struct Reported {
    exit: ExitCode,
    err: String,
}

impl Reported {
    const FAILURE_STARTS: [&'static str; 2] = ["\ncannot ", "\nno Rust sources"];
    const HEADLINE_END: &'static str = ":\n";
    const SITE: &'static str = ".rs:";

    fn clean() -> Reported {
        Reported { exit: ExitCode::SUCCESS, err: String::from("\n") }
    }

    fn headlines(&self) -> usize {
        self.err.matches(Self::HEADLINE_END).count()
    }

    fn failures(&self) -> usize {
        Self::FAILURE_STARTS.iter().map(|start| self.err.matches(start).count()).sum()
    }

    fn violations(&self) -> usize {
        self.err.matches(Self::SITE).count() - self.err.matches(Self::FAILURE_STARTS[0]).count()
    }
}

#[test]
fn a_file_that_does_not_parse_is_reported_without_hiding_other_lints() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "// note\nmod broken;\n")?;
    root.write("src/broken.rs", "\n\nfn (")?;
    let run = root.reported();
    assert_eq!(run.failures(), 1);
    assert!(run.err.contains("cannot parse"));
    assert_eq!(run.violations(), 1);
    Ok(())
}

#[test]
fn a_parse_failure_names_its_line() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod broken;\n")?;
    root.write("src/broken.rs", "struct A;\nstruct B;\nfn f() { let x = (1; }\n")?;
    assert!(root.reported().err.contains("src/broken.rs:3"));
    Ok(())
}

#[test]
fn files_outside_crate_targets_and_fixture_trees_are_not_read() -> Result<(), Failed> {
    let outside = Scratch::package()?;
    outside.write("src/lib.rs", CLEAN)?;
    outside.write("tests/ui/bad.rs", "fn (")?;
    outside.write("tooling/corpus/dirty.rs", DIRTY)?;
    assert_eq!(outside.reported(), Reported::clean());
    let fixtures = Scratch::package()?;
    fixtures.write("src/lib.rs", CLEAN)?;
    fixtures.write("tooling/corpus/case/src/lib.rs", DIRTY)?;
    fixtures.write("tests/fixtures/bad/src/lib.rs", "fn (")?;
    assert_eq!(fixtures.reported(), Reported::clean());
    Ok(())
}

#[test]
fn test_modules_in_their_own_files_are_not_read() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#[cfg(test)]\nmod tests;\n")?;
    root.write("src/tests.rs", "struct S { on: bool }\n")?;
    let run = root.reported();
    assert_eq!(run.violations(), 1);
    assert_eq!(run.err.matches("src/tests.rs").count(), 0);
    Ok(())
}

#[test]
fn a_declared_module_with_no_file_is_a_failure() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod absent;\n")?;
    let run = root.reported();
    assert_eq!(run.failures(), 1);
    assert!(run.err.contains("cannot read"));
    assert!(run.err.contains("src/absent.rs"), "{}", run.err);
    Ok(())
}

#[test]
fn a_tree_with_no_rust_sources_fails_closed() -> Result<(), Failed> {
    let empty = Scratch::temporary()?;
    assert!(empty.reported().err.contains("no Rust sources found"));
    let other = Scratch::temporary()?;
    other.write("README", "x")?;
    assert!(other.reported().err.contains("no Rust sources found"));
    Ok(())
}

#[test]
fn every_path_given_is_checked() -> Result<(), Failed> {
    let scratch = Scratch::temporary()?;
    let roots: Vec<PathBuf> = [("clean", CLEAN), ("dirty", DIRTY)]
        .iter()
        .map(|(package, library)| {
            let manifest = scratch.write_below(package, "Cargo.toml", MANIFEST)?;
            scratch.write_below(package, "src/lib.rs", library)?;
            Ok(manifest.parent().map_or_default(Path::to_path_buf))
        })
        .collect::<Result<Vec<PathBuf>, FileFailure>>()?;
    assert_eq!(Trees::reported_through(&scratch.files, &roots).exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn the_binary_root_is_found_whichever_path_is_given() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/main.rs", "struct Extra;\nfn main() {}\n")?;
    assert_eq!(root.reported().violations(), 1);
    assert_eq!(root.reported_at("src").violations(), 1);
    Ok(())
}

#[test]
fn bin_roots_and_modules_named_like_build_directories_are_read() -> Result<(), Failed> {
    let bins = Scratch::package()?;
    bins.write("src/bin/tool.rs", "struct Extra;\nfn main() {}\n")?;
    bins.write("src/bin/other/main.rs", "mod m {}\nfn main() {}\n")?;
    assert_eq!(bins.reported().violations(), 2);
    let build_named = Scratch::package()?;
    build_named.write("src/lib.rs", "pub mod target;\n")?;
    build_named.write("src/target/mod.rs", DIRTY)?;
    assert_eq!(build_named.reported().violations(), 2);
    Ok(())
}

#[test]
fn symlink_loops_do_not_repeat_findings() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let linked = FakeLinked::new(&root.files, root.at("src/loop"), root.root.clone());
    assert_eq!(Trees::reported_through(&linked, std::slice::from_ref(&root.root)).violations(), 2);
    Ok(())
}

#[test]
fn a_shebang_is_not_a_comment() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#!//usr/bin/env foo\nstruct A;\n")?;
    assert_eq!(root.reported(), Reported::clean());
    Ok(())
}

#[test]
fn a_directory_named_tests_inside_src_is_not_a_package() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "pub mod request;\n")?;
    root.write("src/request.rs", "#[cfg(test)]\nmod tests;\n")?;
    root.write("src/request/tests.rs", "mod a;\n")?;
    root.write("src/request/tests/a.rs", "mod b;\n")?;
    root.write("src/request/tests/a/b.rs", "")?;
    assert_eq!(root.reported().violations(), 1);
    Ok(())
}

#[test]
fn hidden_directories_and_unlisted_crates_are_not_read() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\nexclude = [\"crates/skipped\"]\n")?;
    root.write("crates/one/Cargo.toml", MANIFEST)?;
    root.write("crates/one/src/lib.rs", CLEAN)?;
    root.write("crates/skipped/Cargo.toml", MANIFEST)?;
    root.write("crates/skipped/src/lib.rs", DIRTY)?;
    root.write("vendor/dep/Cargo.toml", MANIFEST)?;
    root.write("vendor/dep/src/lib.rs", DIRTY)?;
    root.write(".claude/worktrees/t1/Cargo.toml", MANIFEST)?;
    root.write(".claude/worktrees/t1/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported(), Reported::clean());
    root.write("crates/two/Cargo.toml", MANIFEST)?;
    root.write("crates/two/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn a_path_dependency_inside_the_workspace_is_read() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"app\"]\n")?;
    root.write("app/Cargo.toml", "[package]\nname = \"app\"\n[dependencies]\nhelp = { path = \"../help\" }\n")?;
    root.write("app/src/lib.rs", CLEAN)?;
    root.write("help/Cargo.toml", "[package]\nname = \"help\"\n")?;
    root.write("help/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn the_verdict_does_not_depend_on_where_the_checkout_lives() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    let flow = "#[test]\nfn t() { for i in [2, 3] { if i > 9 { return; } } }\n";
    let build = "fn main() { println!(\"{}\", 2 + 3); }\n";
    let nested = "pub mod a;\npub mod b;\n";
    let layouts = ["plain/proj", "src/proj", "tests/proj"];
    let projects: Vec<PathBuf> = layouts
        .iter()
        .map(|layout| {
            let manifest = root.write_below(layout, "Cargo.toml", MANIFEST)?;
            root.write_below(layout, "src/lib.rs", nested)?;
            root.write_below(layout, "src/a.rs", "pub struct Tool;\n")?;
            root.write_below(layout, "src/b.rs", "use crate::a::Tool;\npub struct User(Tool);\n")?;
            root.write_below(layout, "tests/flow.rs", flow)?;
            root.write_below(layout, "build.rs", build)?;
            Ok(manifest.parent().map_or_default(Path::to_path_buf))
        })
        .collect::<Result<Vec<PathBuf>, FileFailure>>()?;
    let counts: Vec<usize> = projects
        .iter()
        .map(|project| Trees::reported_through(&root.files, std::slice::from_ref(project)).violations())
        .collect();
    let first = counts.first().copied().unwrap_or_default();
    assert!(first > 0);
    assert!(counts.iter().all(|count| *count == first));
    Ok(())
}

const PATH_CYCLE: &[[&str; 2]] = &[
    ["src/lib.rs", "mod a;\n"],
    ["src/a.rs", "// a\n#[path = \"b.rs\"]\nmod b;\n"],
    ["src/b.rs", "// b\n#[path = \"a.rs\"]\nmod a;\n"],
];
const TWO_ROOTS: &[[&str; 2]] = &[
    ["src/lib.rs", "mod shared;\n"],
    ["src/main.rs", "mod shared;\nfn main() {}\n"],
    ["src/shared.rs", "// shared\n"],
];
const TEST_THEN_MODULE: &[[&str; 2]] = &[
    ["src/lib.rs", "mod a;\n#[cfg(test)]\nmod checks;\n"],
    ["src/a.rs", "#[path = \"checks.rs\"]\nmod checks;\n"],
    ["src/checks.rs", "// checks\n"],
];
const MODULE_THEN_TEST: &[[&str; 2]] = &[
    ["src/lib.rs", "mod shared;\nmod a;\n"],
    ["src/a.rs", "#[cfg(test)]\n#[path = \"shared.rs\"]\nmod again;\n"],
    ["src/shared.rs", "// shared\n"],
];
const TEST_CYCLE: &[[&str; 2]] = &[
    ["src/lib.rs", "#[cfg(test)]\nmod t;\n"],
    ["src/t.rs", "// t\n#[path = \"u.rs\"]\nmod u;\n"],
    ["src/u.rs", "// u\n#[path = \"t.rs\"]\nmod t;\n"],
];

#[test]
fn a_file_reached_twice_or_in_a_cycle_is_read_once() {
    assert_eq!(Scratch::judged(&PATH_CYCLE), Some(2));
    assert_eq!(Scratch::judged(&TWO_ROOTS), Some(2));
    assert_eq!(Scratch::judged(&TEST_THEN_MODULE), Some(1));
    assert_eq!(Scratch::judged(&MODULE_THEN_TEST), Some(2));
    assert_eq!(Scratch::judged(&TEST_CYCLE), Some(1));
}

#[test]
fn a_crate_is_named_by_its_manifest_not_its_directory() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n")?;
    root.write("crates/engine-impl/Cargo.toml", "[package]\nname = \"engine\"\n")?;
    root.write("crates/engine-impl/src/lib.rs", "mod x;\npub use x::{Used, Unused};\n")?;
    root.write(
        "crates/engine-impl/src/x.rs",
        "pub struct Used;\nimpl Used {\n    pub fn lift(&self) {}\n}\npub struct Unused;\nimpl Unused {\n    pub fn lift(&self) {}\n}\n",
    )?;
    root.write("crates/app/Cargo.toml", "[package]\nname = \"app\"\n")?;
    root.write("crates/app/src/lib.rs", "pub use engine::Used;\n")?;
    assert_eq!(root.reported().violations(), 1);
    Ok(())
}

#[test]
fn inline_modules_and_path_attributes_lead_to_the_file_read() -> Result<(), Failed> {
    let layouts = [
        ("mod outer {\n    mod inner;\n}\n", "src/outer/inner.rs"),
        ("#[path = \"elsewhere/place.rs\"]\nmod renamed;\n", "src/elsewhere/place.rs"),
    ];
    layouts.iter().try_for_each(|(library, module)| {
        let root = Scratch::package()?;
        root.write("src/lib.rs", library)?;
        root.write(module, DIRTY)?;
        assert_eq!(root.reported().violations(), 2);
        Ok(())
    })
}

#[test]
fn targets_declared_with_a_path_are_read() -> Result<(), Failed> {
    let declared = [
        ("[[test]]\nname = \"x\"\npath = \"check/x.rs\"\n", "check/x.rs"),
        ("[[example]]\nname = \"e\"\npath = \"alt/e.rs\"\n", "alt/e.rs"),
        ("[[bench]]\nname = \"b\"\npath = \"alt/b.rs\"\n", "alt/b.rs"),
        ("[[bin]]\nname = \"c\"\npath = \"alt/c.rs\"\n", "alt/c.rs"),
    ];
    declared.iter().try_for_each(|(table, file)| {
        let root = Scratch::temporary()?;
        root.write("Cargo.toml", &format!("{MANIFEST}{table}"))?;
        root.write(file, DIRTY)?;
        assert_eq!(root.reported().violations(), 2, "{file}");
        Ok(())
    })
}

#[test]
fn an_example_that_is_not_run_as_a_test_is_not_a_skipped_test() -> Result<(), Failed> {
    let declared = |kind: &str, file: &str| -> Result<Reported, FileFailure> {
        let root = Scratch::temporary()?;
        let table = format!("{MANIFEST}[[{kind}]]\nname = \"e\"\ntest = false\n");
        root.write("Cargo.toml", &table)?;
        root.write(file, CLEAN)?;
        Ok(root.reported())
    };
    assert_eq!(declared("example", "examples/e.rs")?, Reported::clean());
    assert_eq!(declared("test", "tests/e.rs")?.exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn conditional_paths_and_nested_packages_are_read() -> Result<(), Failed> {
    let conditional = Scratch::package()?;
    conditional.write("src/lib.rs", "#[cfg_attr(unix, path = \"b.rs\")]\nmod a;\n")?;
    conditional.write("src/a.rs", CLEAN)?;
    conditional.write("src/b.rs", DIRTY)?;
    assert_eq!(conditional.reported().violations(), 2);
    let nested = Scratch::package()?;
    nested.write("src/lib.rs", CLEAN)?;
    nested.write("inner/Cargo.toml", "[package]\nname = \"inner\"\n")?;
    nested.write("inner/src/lib.rs", DIRTY)?;
    assert_eq!(nested.reported().violations(), 2);
    Ok(())
}

#[test]
fn a_package_with_its_own_workspace_inside_another_workspace_is_read() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = []\n")?;
    root.write("nested/Cargo.toml", "[package]\nname = \"nested\"\n[workspace]\n")?;
    root.write("nested/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn hidden_and_build_directories_hide_their_packages() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    let copies: Vec<PathBuf> = ["target/package/copy", ".cache/copy"]
        .iter()
        .map(|copy| {
            root.write(&format!("{copy}/Cargo.toml"), "[package]\nname = \"copy\"\n[workspace]\n")?;
            root.write(&format!("{copy}/src/lib.rs"), DIRTY)?;
            Ok(root.at(copy))
        })
        .collect::<Result<Vec<PathBuf>, FileFailure>>()?;
    assert_eq!(root.reported(), Reported::clean());
    let direct: Vec<usize> = copies
        .iter()
        .map(|copy| Trees::reported_through(&root.files, std::slice::from_ref(copy)).violations())
        .collect();
    assert_eq!(direct, vec![2, 2]);
    Ok(())
}

#[test]
fn a_workspace_manifest_is_not_a_package() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    root.write("src/lib.rs", DIRTY)?;
    root.write("one/Cargo.toml", MANIFEST)?;
    root.write("one/src/lib.rs", CLEAN)?;
    assert_eq!(root.reported(), Reported::clean());
    Ok(())
}

#[test]
fn a_manifest_that_cannot_be_read_fails_closed() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.directory("Cargo.toml")?;
    assert!(root.reported().err.contains("Cargo.toml"));
    Ok(())
}

#[test]
fn a_manifest_beside_the_linted_member_that_cannot_be_read_fails_closed() -> Result<(), Failed> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    root.write("one/Cargo.toml", MANIFEST)?;
    root.write("one/src/lib.rs", CLEAN)?;
    root.write("two/src/lib.rs", CLEAN)?;
    assert_eq!(root.reported_at("one"), Reported::clean());
    root.directory("two/Cargo.toml")?;
    assert!(root.reported_at("one").err.contains("two/Cargo.toml"));
    Ok(())
}

#[test]
fn a_directory_that_cannot_be_listed_fails_closed_wherever_it_is_reached() -> Result<(), Failed> {
    let scratch = Scratch::temporary()?;
    scratch.write("workspace/Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    scratch.write("workspace/one/Cargo.toml", MANIFEST)?;
    scratch.write("workspace/one/src/lib.rs", CLEAN)?;
    scratch.write("package/Cargo.toml", MANIFEST)?;
    scratch.write("package/src/lib.rs", CLEAN)?;
    let cases = [
        ("workspace", "one/src/inner", "one"),
        ("workspace", "one/.hidden", "one"),
        ("workspace", "two", "one"),
        ("package", "tests", "src"),
    ];
    cases.iter().try_for_each(|(root, directory, given)| {
        scratch.write(&format!("{root}/{directory}/kept.txt"), "x")?;
        let unlistable = scratch.files.canonical(&scratch.at(&format!("{root}/{directory}")))?;
        assert_eq!(scratch.reported_at(&format!("{root}/{given}")), Reported::clean(), "{directory}");
        let trees = [scratch.at(&format!("{root}/{given}"))];
        let run = Trees::reported_through(&FakeUnlistable::new(&scratch.files, unlistable.clone()), &trees);
        assert_eq!(run.exit, ExitCode::FAILURE, "{directory}");
        assert!(run.err.contains(&format!("cannot read {}", unlistable.display())), "{directory}: {}", run.err);
        Ok(())
    })
}

#[test]
fn a_twenty_thousand_link_method_chain_is_linted_without_overflowing_the_stack() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", &format!("fn long(x: u8) {{ x{} }}\n", ".m()".repeat(LONG_CHAIN_LINKS)))?;
    let run = root.reported();
    assert!(run.err.contains("src/lib.rs:1"));
    Ok(())
}

#[test]
fn source_nested_past_what_anyone_writes_is_refused_not_parsed() -> Result<(), Failed> {
    let shapes = [
        format!("fn render_page() {{ let a = {}1{}; }}\n", "(".repeat(DEEP_NESTING), ")".repeat(DEEP_NESTING)),
        format!("fn render_page() {}{}\n", "{".repeat(DEEP_NESTING), "}".repeat(DEEP_NESTING)),
        format!("fn render_page(a: {}u8{}) {{}}\n", "A<".repeat(DEEP_NESTING), ">".repeat(DEEP_NESTING)),
        format!("{}{}\n", "mod a { ".repeat(DEEP_NESTING), "}".repeat(DEEP_NESTING)),
        format!("fn render_page() {{ let a = {}1{}; }}\n", "vec![".repeat(DEEP_NESTING), "]".repeat(DEEP_NESTING)),
    ];
    shapes.iter().try_for_each(|shape| {
        let root = Scratch::package()?;
        root.write("src/lib.rs", shape)?;
        let run = root.reported();
        assert_eq!(run.failures(), 1);
        assert!(run.err.contains("src/lib.rs:1"));
        assert!(run.err.contains("nested more than"));
        Ok(())
    })
}

#[test]
fn deep_but_ordinary_nesting_is_still_linted() -> Result<(), Failed> {
    let root = Scratch::package()?;
    let nested = format!(
        "fn render_page() {{ let a = {}1{}; }}\n// note\n",
        "(".repeat(SHALLOW_NESTING),
        ")".repeat(SHALLOW_NESTING)
    );
    root.write("src/lib.rs", &nested)?;
    let run = root.reported();
    assert_eq!(run.failures(), 0);
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn nesting_is_counted_where_it_is_open_not_in_total() -> Result<(), Failed> {
    let (open, close) = ("(".repeat(MIDDLING_NESTING), ")".repeat(MIDDLING_NESTING));
    let (angle, shut) = ("A<".repeat(MIDDLING_NESTING), ">".repeat(MIDDLING_NESTING));
    let cases = [
        (format!("fn render_page() {{ let a = {open}1{close}; let b = {open}1{close}; }}\n"), 0),
        (format!("fn render_page(a: {angle}u8{shut}, b: {angle}u8{shut}) {{}}\n"), 0),
        (format!("fn render_page(a: {angle}u8;{angle}u8) {{}}\n"), 0),
        (format!("fn render_page() {{ let a = {open}){}; }}\n", "(".repeat(SHALLOW_NESTING)), 1),
        (
            format!(
                "fn render_page(a: {angle}u8{}{}) {{}}\n",
                "->".repeat(SHALLOW_NESTING),
                "A<".repeat(SHALLOW_NESTING)
            ),
            1,
        ),
        (format!("fn render_page(a: {angle}{}{angle}) {{}}\n", ">".repeat(SHALLOW_NESTING)), 1),
    ];
    cases.iter().try_for_each(|(source, failures)| {
        let root = Scratch::package()?;
        root.write("src/lib.rs", source)?;
        assert_eq!(root.reported().err.matches("nested more than").count(), *failures, "{source}");
        Ok(())
    })
}

#[test]
fn a_tree_read_by_many_workers_reports_the_same_each_time() -> Result<(), Failed> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod first;\nmod second;\n#[cfg(test)]\nmod tests;\n")?;
    root.write("src/tests.rs", DIRTY)?;
    ["first", "second"].iter().try_for_each(|name| {
        root.write(&format!("src/{name}.rs"), "mod inner;\n#[cfg(test)]\nmod tests;\n")?;
        root.write(&format!("src/{name}/tests.rs"), DIRTY)?;
        root.write(&format!("src/{name}/inner.rs"), DIRTY).map(|_| ())
    })?;
    let first = root.reported().err;
    let runs: Vec<String> = std::iter::repeat_with(|| root.reported().err).take(8).collect();
    assert!(runs.iter().all(|run| *run == first), "{runs:?}");
    assert!(first.contains("src/first/inner.rs"), "{first}");
    Ok(())
}
