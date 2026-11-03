use std::error::Error;

use lint_core::Lint;
use lint_crate_graph::CrateGraph;
use lint_fixtures::{Lines, Sources};
use lint_reader::SourceFiles;
use lint_runner::Cli;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, FileFailure, Files, Kind, Memory};

use crate::fakes::{FakeLinked, FakeUnlistable};

const DIRTY: &str = "// note\nfn render_page() {}\n";
const CLEAN: &str = "struct A;\n";
const PAIRED: &str = "pub struct A;\nimpl A {\n    pub fn open(&self) {}\n    pub fn close(&self) {}\n}\n";
const REPEATED: &str = "pub struct A;\nimpl A {\n    pub fn a(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 3 {\n            total += item * 2;\n        } else {\n            total += 1;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n}\n";
const REPEATED_AGAIN: &str = "pub struct B;\nimpl B {\n    pub fn b(rows: &[u32]) -> u32 {\n    let mut sum = 0;\n    for row in rows {\n        if *row > 3 {\n            sum += row * 2;\n        } else {\n            sum += 1;\n        }\n    }\n    if sum > 100 {\n        return 100;\n    }\n    sum\n}\n}\n";
const INDEX: &str = "pub trait Index {\n    fn search(&self, query: &Query) -> Vec<Hit>;\n}\n";
const MODULES: &str = "mod one;\nmod two;\n";
const CHAIN_A: &str =
    "pub struct A0(A1);\npub struct A1(A2);\npub struct A2(A3);\npub struct A3(A4);\npub struct A4;\n";
const CHAIN_B: &str =
    "pub struct B0(B1);\npub struct B1(B2);\npub struct B2(B3);\npub struct B3(B4);\npub struct B4;\n";
const SHORT_CHAIN_B: &str = "pub struct B0(B1);\npub struct B1(B2);\npub struct B2(B3);\npub struct B3;\n";
const LONG_CHAIN_LINKS: usize = 20_000;
const DEEP_NESTING: usize = 5_000;
const SHALLOW_NESTING: usize = 100;
const MIDDLING_NESTING: usize = 200;
const MANIFEST: &str = "[package]\nname = \"r\"\n";

#[test]
fn every_lint_reports_over_the_same_files() -> Result<(), Box<dyn Error>> {
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
fn the_runner_runs_every_lint_in_the_workspace() -> Result<(), Box<dyn Error>> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let workspace = golden::fixtures!("..")?;
    let implemented = workspace.entries(&crates)?.iter().try_fold(0, |found, entry| {
        let source = entry.join("src");
        Ok::<usize, FileFailure>(
            found
                + Trees::occurrences(&workspace, &source, "impl Lint for")?
                + Trees::occurrences(&workspace, &source, "impl GraphLint for")?,
        )
    })?;
    let listed_by_the_mode = include_str!("../src/mode.rs").matches("::report(files)").count();
    let runner = include_str!("../src/lints.rs");
    let listed_by_the_runner =
        ["::report(files)", "flows.report::<"].iter().map(|call| runner.matches(call).count()).sum::<usize>();
    assert!(listed_by_the_mode < implemented);
    let elegance = include_str!("../src/elegance.rs");
    let listed_by_elegance = ["::report(files)", "::report_facts(&facts)", "::report_graph(&facts, &transitions)"]
        .iter()
        .map(|call| elegance.matches(call).count())
        .sum::<usize>();
    assert_eq!(listed_by_the_runner, implemented - listed_by_the_mode - listed_by_elegance);
    Ok(())
}

#[test]
fn a_tree_with_violations_is_not_clean() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    root.write("src/clean.rs", CLEAN)?;
    let run = root.reported();
    assert_eq!(run.headlines(), 2);
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn a_clean_tree_is_clean_and_build_directories_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    root.write("target/out.rs", DIRTY)?;
    assert!(root.reported().is_clean());
    Ok(())
}

#[test]
fn a_report_only_finding_blocks_nothing() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "pub struct A;\n")?;
    root.write("src/paired.rs", PAIRED)?;
    assert!(root.reported().is_clean());
    assert_eq!(root.reported().exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn the_report_flag_prints_elegance_reports_and_blocks_nothing() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let run = root.flagged(&["--report"]);
    assert_eq!(run.exit, ExitCode::SUCCESS);
    assert!(run.out.contains("/src/lib.rs:2: crate r: 1 functions by complexity"), "{}", run.out);
    assert_eq!(run.err, "");
    Ok(())
}

#[test]
fn without_the_report_flag_elegance_reports_stay_off() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let run = root.flagged(&[]);
    assert_eq!(run.out.matches("functions by complexity").count(), 0, "{}", run.out);
    Ok(())
}

#[test]
fn a_repeated_shape_fails_the_run() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", MODULES)?;
    root.write("src/one.rs", REPEATED)?;
    root.write("src/two.rs", REPEATED_AGAIN)?;
    let run = root.flagged(&[]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains("/src/one.rs:3: "), "{}", run.err);
    assert!(run.err.contains("2 shipped copies, also at "), "{}", run.err);
    assert!(run.err.contains("/src/two.rs:3; "), "{}", run.err);
    Ok(())
}

#[test]
fn the_report_flag_fails_on_an_unreadable_root() {
    let run = Trees::flagged(&Memory::default(), Path::new("/no/such/root"), &["--report"]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert_ne!(run.err, "");
}

#[test]
fn the_query_command_lists_matching_abstractions_and_blocks_nothing() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", &format!("{DIRTY}{INDEX}"))?;
    let run = root.flagged(&["query", "index", "--role", "fn f(&self, q: &Q) -> Vec<Hit>", "--in"]);
    assert_eq!(run.exit, ExitCode::SUCCESS);
    let lines: Vec<&str> = text_lines::Lines::from(run.out.as_str()).collect();
    assert_eq!(
        lines.first().map(|line| line.contains("score 5: trait Index in r at /scratch/src/lib.rs:3")),
        Some(true)
    );
    assert_eq!(lines.get(1).map(|line| line.contains("search fn(&self, _) -> many")), Some(true));
    assert_eq!(run.err, "");
    Ok(())
}

#[test]
fn the_query_command_refuses_a_malformed_role() {
    let run = Trees::flagged(&Memory::default(), Path::new("/scratch"), &["query", "--role", "nonsense", "--in"]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains("is not a Rust signature"), "{}", run.err);
    assert_eq!(run.out, "");
}

#[test]
fn a_violation_fails_the_run() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().exit, ExitCode::FAILURE);
    Ok(())
}

struct Trees;

impl Trees {
    fn occurrences(files: &impl Files, path: &Path, needle: &str) -> Result<usize, FileFailure> {
        match files.kind(path) {
            Kind::Directory => files
                .entries(path)?
                .iter()
                .try_fold(0, |found, entry| Ok(found + Self::occurrences(files, entry, needle)?)),
            _ => Ok(files.read_text(path).map_or(0, |text| text.as_ref().matches(needle).count())),
        }
    }

    fn flagged(files: &(impl Files + Sync), root: &Path, flags: &[&str]) -> Printed {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let arguments = flags.iter().map(OsString::from).chain([OsString::from(root)]);
        let exit = Cli::from_args(arguments).report_to(files, &mut out, &mut err);
        Printed {
            exit,
            out: String::from_utf8(out).unwrap_or_default(),
            err: String::from_utf8(err).unwrap_or_default(),
        }
    }

    fn reported_through(files: &(impl Files + Sync), roots: &[PathBuf]) -> Reported {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let exit = Cli::from_args(roots.iter().map(OsString::from)).report_to(files, &mut out, &mut err);
        Reported { exit, err: format!("\n{}", String::from_utf8(err).unwrap_or_default()) }
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

    fn write(&self, path: &str, text: &str) -> Result<(), FileFailure> {
        let file = self.at(path);
        file.parent().map_or(Ok(()), |parent| self.files.create_directories(parent))?;
        self.files.write(&file, Bytes::from(text))
    }

    fn write_below(&self, directory: &str, path: &str, text: &str) -> Result<(), FileFailure> {
        self.write(&format!("{directory}/{path}"), text)
    }

    fn directory(&self, path: &str) -> Result<(), FileFailure> {
        self.files.create_directories(&self.at(path))
    }

    fn reported(&self) -> Reported {
        Trees::reported_through(&self.files, std::slice::from_ref(&self.root))
    }

    fn reported_at(&self, below: &str) -> Reported {
        Trees::reported_through(&self.files, &[self.at(below)])
    }

    fn flagged(&self, flags: &[&str]) -> Printed {
        Trees::flagged(&self.files, &self.root, flags)
    }

    fn crate_graph_lines(&self) -> Result<Lines, Box<dyn Error>> {
        let readout = SourceFiles::read(&self.files, &self.root)?;
        Ok(Sources::lines(&CrateGraph::report(readout.files())))
    }
}

struct Printed {
    exit: ExitCode,
    out: String,
    err: String,
}

struct Reported {
    exit: ExitCode,
    err: String,
}

impl Reported {
    const FAILURE_STARTS: [&'static str; 2] = ["\ncannot ", "\nno Rust sources"];
    const HEADLINE_END: &'static str = ":\n";
    const SITE: &'static str = ".rs:";

    fn is_clean(&self) -> bool {
        (self.exit, self.err.as_str()) == (ExitCode::SUCCESS, "\n")
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
fn a_file_that_does_not_parse_is_reported_without_hiding_other_lints() -> Result<(), Box<dyn Error>> {
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
fn a_parse_failure_names_its_line() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod broken;\n")?;
    root.write("src/broken.rs", "struct A;\nstruct B;\nfn f() { let x = (1; }\n")?;
    assert!(root.reported().err.contains("src/broken.rs:3"));
    Ok(())
}

#[test]
fn files_outside_crate_targets_and_fixture_trees_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    root.write("tests/ui/bad.rs", "fn (")?;
    root.write("tooling/corpus/dirty.rs", DIRTY)?;
    assert!(root.reported().is_clean());
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    root.write("tooling/corpus/case/src/lib.rs", DIRTY)?;
    root.write("tests/fixtures/bad/src/lib.rs", "fn (")?;
    assert!(root.reported().is_clean());
    Ok(())
}

#[test]
fn test_modules_in_their_own_files_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#[cfg(test)]\nmod tests;\n")?;
    root.write("src/tests.rs", "struct S { on: bool }\n")?;
    let run = root.reported();
    assert_eq!(run.violations(), 1);
    assert_eq!(run.err.matches("src/tests.rs").count(), 0);
    Ok(())
}

#[test]
fn a_declared_module_with_no_file_is_a_failure() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "mod absent;\n")?;
    let run = root.reported();
    assert_eq!(run.failures(), 1);
    assert!(run.err.contains("cannot read"));
    assert!(run.err.contains("src/absent.rs"), "{}", run.err);
    Ok(())
}

#[test]
fn a_tree_with_no_rust_sources_fails_closed() -> Result<(), Box<dyn Error>> {
    let empty = Scratch::temporary()?;
    assert!(empty.reported().err.contains("no Rust sources found"));
    let other = Scratch::temporary()?;
    other.write("README", "x")?;
    assert!(other.reported().err.contains("no Rust sources found"));
    Ok(())
}

#[test]
fn every_path_given_is_checked() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::temporary()?;
    [("clean", CLEAN), ("dirty", DIRTY)].iter().try_for_each(|(package, library)| {
        scratch.write_below(package, "Cargo.toml", MANIFEST)?;
        scratch.write_below(package, "src/lib.rs", library)
    })?;
    let roots = [scratch.at("clean"), scratch.at("dirty")];
    assert_eq!(Trees::reported_through(&scratch.files, &roots).exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn the_binary_root_is_found_whichever_path_is_given() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/main.rs", "struct Extra;\nfn main() {}\n")?;
    assert_eq!(root.reported().violations(), 1);
    assert_eq!(root.reported_at("src").violations(), 1);
    Ok(())
}

#[test]
fn bin_roots_and_modules_named_like_build_directories_are_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/bin/tool.rs", "struct Extra;\nfn main() {}\n")?;
    root.write("src/bin/other/main.rs", "mod m {}\nfn main() {}\n")?;
    assert_eq!(root.reported().violations(), 2);
    let root = Scratch::package()?;
    root.write("src/lib.rs", "pub mod target;\n")?;
    root.write("src/target/mod.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn symlink_loops_do_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", DIRTY)?;
    let linked = FakeLinked::new(&root.files, root.at("src/loop"), root.root.clone());
    assert_eq!(Trees::reported_through(&linked, std::slice::from_ref(&root.root)).violations(), 2);
    Ok(())
}

#[test]
fn a_shebang_is_not_a_comment() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#!//usr/bin/env foo\nstruct A;\n")?;
    assert!(root.reported().is_clean());
    Ok(())
}

#[test]
fn a_directory_named_tests_inside_src_is_not_a_package() -> Result<(), Box<dyn Error>> {
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
fn hidden_directories_and_unlisted_crates_are_not_read() -> Result<(), Box<dyn Error>> {
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
    assert!(root.reported().is_clean());
    root.write("crates/two/Cargo.toml", MANIFEST)?;
    root.write("crates/two/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn a_path_dependency_inside_the_workspace_is_read() -> Result<(), Box<dyn Error>> {
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
fn the_verdict_does_not_depend_on_where_the_checkout_lives() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    let flow = "#[test]\nfn t() { for i in [2, 3] { if i > 9 { return; } } }\n";
    let build = "fn main() { println!(\"{}\", 2 + 3); }\n";
    let nested = "pub mod a;\npub mod b;\n";
    let layouts = ["plain/proj", "src/proj", "tests/proj"];
    layouts.iter().try_for_each(|layout| {
        root.write_below(layout, "Cargo.toml", MANIFEST)?;
        root.write_below(layout, "src/lib.rs", nested)?;
        root.write_below(layout, "src/a.rs", "pub struct Tool;\n")?;
        root.write_below(layout, "src/b.rs", "use crate::a::Tool;\npub struct User(Tool);\n")?;
        root.write_below(layout, "tests/flow.rs", flow)?;
        root.write_below(layout, "build.rs", build)
    })?;
    let counts: Vec<usize> = layouts.iter().map(|layout| root.reported_at(layout).violations()).collect();
    let first = counts.first().copied().unwrap_or_default();
    assert!(first > 0);
    assert!(counts.iter().all(|count| *count == first));
    Ok(())
}

#[test]
fn a_module_used_by_a_sibling_and_its_test_file_stays_put() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "pub mod a;\n")?;
    root.write("src/a.rs", "mod helper;\npub mod user;\n#[cfg(test)]\nmod tests;\n")?;
    root.write("src/a/helper.rs", "pub struct Help;\nimpl Help {\n    pub fn lift(&self) {}\n}\n")?;
    root.write("src/a/user.rs", "use super::helper::Help;\npub struct User(Help);\n")?;
    root.write("src/a/tests.rs", "use super::helper::Help;\n")?;
    assert_eq!(root.reported().violations(), 1);
    root.write("src/a/tests.rs", "")?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn a_crate_is_named_by_its_manifest_not_its_directory() -> Result<(), Box<dyn Error>> {
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
fn a_library_and_its_binary_are_judged_apart() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CHAIN_A)?;
    let main = format!("use r::A0;\n{SHORT_CHAIN_B}fn main() {{\n    let _ = (B0, A0);\n}}\n");
    root.write("src/main.rs", &main)?;
    assert_eq!(root.crate_graph_lines()?, Lines::default());
    Ok(())
}

#[test]
fn a_library_outside_src_is_judged_and_named_by_its_root_file() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", &format!("{MANIFEST}[lib]\npath = \"code/lib.rs\"\n"))?;
    root.write("code/lib.rs", &format!("pub mod a;\n{CHAIN_A}"))?;
    root.write("code/a.rs", CHAIN_B)?;
    let named = format!("{}:1", root.at("code/lib.rs").display());
    assert_eq!(root.crate_graph_lines()?, [named.as_str()]);
    Ok(())
}

#[test]
fn inline_modules_and_path_attributes_lead_to_the_file_read() -> Result<(), Box<dyn Error>> {
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
fn targets_declared_with_a_path_are_read() -> Result<(), Box<dyn Error>> {
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
fn an_example_that_is_not_run_as_a_test_is_not_a_skipped_test() -> Result<(), Box<dyn Error>> {
    let declared = [("example", "examples/e.rs", true), ("test", "tests/e.rs", false)];
    declared.iter().try_for_each(|(kind, file, clean)| {
        let root = Scratch::temporary()?;
        let table = format!("{MANIFEST}[[{kind}]]\nname = \"e\"\ntest = false\n");
        root.write("Cargo.toml", &table)?;
        root.write(file, CLEAN)?;
        assert_eq!(root.reported().is_clean(), *clean, "{kind}");
        Ok(())
    })
}

#[test]
fn conditional_paths_and_nested_packages_are_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", "#[cfg_attr(unix, path = \"b.rs\")]\nmod a;\n")?;
    root.write("src/a.rs", CLEAN)?;
    root.write("src/b.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    root.write("inner/Cargo.toml", "[package]\nname = \"inner\"\n")?;
    root.write("inner/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn a_package_with_its_own_workspace_inside_another_workspace_is_read() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = []\n")?;
    root.write("nested/Cargo.toml", "[package]\nname = \"nested\"\n[workspace]\n")?;
    root.write("nested/src/lib.rs", DIRTY)?;
    assert_eq!(root.reported().violations(), 2);
    Ok(())
}

#[test]
fn hidden_and_build_directories_hide_their_packages() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", CLEAN)?;
    ["target/package/copy", ".cache/copy"].iter().try_for_each(|copy| {
        root.write(&format!("{copy}/Cargo.toml"), "[package]\nname = \"copy\"\n[workspace]\n")?;
        root.write(&format!("{copy}/src/lib.rs"), DIRTY)
    })?;
    assert!(root.reported().is_clean());
    Ok(())
}

#[test]
fn a_workspace_manifest_is_not_a_package() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    root.write("src/lib.rs", DIRTY)?;
    root.write("one/Cargo.toml", MANIFEST)?;
    root.write("one/src/lib.rs", CLEAN)?;
    assert!(root.reported().is_clean());
    Ok(())
}

#[test]
fn a_manifest_that_cannot_be_read_fails_closed() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    root.directory("Cargo.toml")?;
    assert!(root.reported().err.contains("Cargo.toml"));
    Ok(())
}

#[test]
fn a_manifest_beside_the_linted_member_that_cannot_be_read_fails_closed() -> Result<(), Box<dyn Error>> {
    let root = Scratch::temporary()?;
    root.write("Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    root.write("one/Cargo.toml", MANIFEST)?;
    root.write("one/src/lib.rs", CLEAN)?;
    root.write("two/src/lib.rs", CLEAN)?;
    assert!(root.reported_at("one").is_clean());
    root.directory("two/Cargo.toml")?;
    assert!(root.reported_at("one").err.contains("two/Cargo.toml"));
    Ok(())
}

#[test]
fn a_directory_that_cannot_be_listed_fails_closed_wherever_it_is_reached() -> Result<(), Box<dyn Error>> {
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
        assert!(scratch.reported_at(&format!("{root}/{given}")).is_clean(), "{directory}");
        let given = [scratch.at(&format!("{root}/{given}"))];
        let run = Trees::reported_through(&FakeUnlistable::new(&scratch.files, unlistable.clone()), &given);
        assert_eq!(run.exit, ExitCode::FAILURE, "{directory}");
        assert!(run.err.contains(&format!("cannot read {}", unlistable.display())), "{directory}: {}", run.err);
        Ok(())
    })
}

#[test]
fn a_twenty_thousand_link_method_chain_is_linted_without_overflowing_the_stack() -> Result<(), Box<dyn Error>> {
    let root = Scratch::package()?;
    root.write("src/lib.rs", &format!("fn long(x: u8) {{ x{} }}\n", ".m()".repeat(LONG_CHAIN_LINKS)))?;
    let run = root.reported();
    assert!(run.err.contains("src/lib.rs:1"));
    Ok(())
}

#[test]
fn source_nested_past_what_anyone_writes_is_refused_not_parsed() -> Result<(), Box<dyn Error>> {
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
fn deep_but_ordinary_nesting_is_still_linted() -> Result<(), Box<dyn Error>> {
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
fn nesting_is_counted_where_it_is_open_not_in_total() -> Result<(), Box<dyn Error>> {
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
        (format!("fn render_page(a: {angle}{}) {{}}\n", ">".repeat(SHALLOW_NESTING) + &angle), 1),
    ];
    cases.iter().try_for_each(|(source, failures)| {
        let root = Scratch::package()?;
        root.write("src/lib.rs", source)?;
        assert_eq!(
            root.reported().err.matches("nested more than").count(),
            *failures,
            "{}",
            &source[..source.len().min(40)]
        );
        Ok(())
    })
}
