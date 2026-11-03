use std::error::Error;

use lint_core::Lint;
use lint_crate_graph::CrateGraph;
use lint_fixtures::{Lines, Sources};
use lint_reader::SourceFiles;
use lint_runner::Cli;
use ports::Disk;

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, FileFailure, Files, Kind, Text};

const DIRTY: &str = "// note\nfn render_page() {}\n";
const CLEAN: &str = "struct A;\n";
const PAIRED: &str = "pub struct A;\nimpl A {\n    pub fn open(&self) {}\n    pub fn close(&self) {}\n}\n";
const REPEATED: &str = "pub struct A;\nimpl A {\n    pub fn a(items: &[u32]) -> u32 {\n    let mut total = 0;\n    for item in items {\n        if *item > 3 {\n            total += item * 2;\n        } else {\n            total += 1;\n        }\n    }\n    if total > 100 {\n        return 100;\n    }\n    total\n}\n}\n";
const REPEATED_AGAIN: &str = "pub struct B;\nimpl B {\n    pub fn b(rows: &[u32]) -> u32 {\n    let mut sum = 0;\n    for row in rows {\n        if *row > 3 {\n            sum += row * 2;\n        } else {\n            sum += 1;\n        }\n    }\n    if sum > 100 {\n        return 100;\n    }\n    sum\n}\n}\n";
const MODULES: &str = "mod one;\nmod two;\n";
const LONG_CHAIN_LINKS: usize = 20_000;
const DEEP_NESTING: usize = 5_000;
const SHALLOW_NESTING: usize = 100;
const MIDDLING_NESTING: usize = 200;
const MANIFEST: &str = "[package]\nname = \"r\"\n";

#[test]
fn every_lint_reports_over_the_same_files() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", DIRTY)?;
    let run = Trees::reported(root.path());
    assert_eq!(run.violations(), 2);
    assert_eq!(run.headlines(), 2);
    assert!(run.err.contains("src/lib.rs:1"));
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn the_runner_runs_every_lint_in_the_workspace() -> Result<(), Box<dyn Error>> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut implemented = 0;
    for entry in Disk::entries(&here.join(".."))? {
        implemented += Disk::occurrences(&entry.join("src"), "impl Lint for")?;
    }
    let listed_by_the_cli = Disk::occurrences(&here.join("src/cli.rs"), "::report(files)")?;
    let listed_by_the_runner = Disk::occurrences(&here.join("src/lints.rs"), "::report(files)")?;
    assert!(listed_by_the_cli < implemented);
    let listed_by_elegance = Disk::occurrences(&here.join("src/elegance.rs"), "::report(files)")?;
    assert_eq!(listed_by_the_runner, implemented - listed_by_the_cli - listed_by_elegance);
    Ok(())
}

#[test]
fn a_tree_with_violations_is_not_clean() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", DIRTY)?;
    Disk::write(root.path(), "src/clean.rs", CLEAN)?;
    let run = Trees::reported(root.path());
    assert_eq!(run.headlines(), 2);
    assert!(run.err.contains("src/lib.rs:2"));
    Ok(())
}

#[test]
fn a_clean_tree_is_clean_and_build_directories_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "target/out.rs", DIRTY)?;
    assert!(Trees::reported(root.path()).is_clean());
    Ok(())
}

#[test]
fn a_report_only_finding_blocks_nothing() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "pub struct A;\n")?;
    Disk::write(root.path(), "src/paired.rs", PAIRED)?;
    assert!(Trees::reported(root.path()).is_clean());
    assert_eq!(Trees::reported(root.path()).exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn the_report_flag_prints_elegance_reports_and_blocks_nothing() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", &format!("{DIRTY}{MODULES}"))?;
    Disk::write(root.path(), "src/one.rs", REPEATED)?;
    Disk::write(root.path(), "src/two.rs", REPEATED_AGAIN)?;
    let run = Trees::flagged(root.path(), &["--report"]);
    assert_eq!(run.exit, ExitCode::SUCCESS);
    assert!(run.out.contains("/src/one.rs:3: "), "{}", run.out);
    assert!(run.out.contains("2 copies, also at ") && run.out.contains("/src/two.rs:3; "), "{}", run.out);
    assert_eq!(run.err, "");
    Ok(())
}

#[test]
fn without_the_report_flag_elegance_reports_stay_off() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", MODULES)?;
    Disk::write(root.path(), "src/one.rs", REPEATED)?;
    Disk::write(root.path(), "src/two.rs", REPEATED_AGAIN)?;
    let run = Trees::flagged(root.path(), &[]);
    assert!(!run.out.contains("copies"), "{}", run.out);
    Ok(())
}

#[test]
fn the_report_flag_fails_on_an_unreadable_root() {
    let run = Trees::flagged(Path::new("/no/such/root"), &["--report"]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert_ne!(run.err, "");
}

#[test]
fn a_violation_fails_the_run() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).exit, ExitCode::FAILURE);
    Ok(())
}

struct Trees;

impl Trees {
    fn chain(prefix: &str, size: usize) -> String {
        let lines: Vec<String> = (0..size)
            .map(|index| {
                if index + 1 < size {
                    format!("pub struct {prefix}{index}({prefix}{});\n", index + 1)
                } else {
                    format!("pub struct {prefix}{index};\n")
                }
            })
            .collect();
        lines.concat()
    }

    fn crate_graph_lines(root: &Path) -> Result<Lines, Box<dyn Error>> {
        let readout = SourceFiles::read(&file_ports::Disk, root)?;
        Ok(Sources::lines(&CrateGraph::report(readout.files())))
    }

    fn flagged(root: &Path, flags: &[&str]) -> Printed {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let arguments = flags.iter().map(OsString::from).chain([OsString::from(root)]);
        let exit = Cli::from_args(arguments).report_to(&file_ports::Disk, &mut out, &mut err);
        Printed {
            exit,
            out: String::from_utf8(out).unwrap_or_default(),
            err: String::from_utf8(err).unwrap_or_default(),
        }
    }

    fn reported(root: &Path) -> Reported {
        Self::reported_all(&[root.to_path_buf()])
    }

    fn reported_all(roots: &[PathBuf]) -> Reported {
        Self::reported_through(&file_ports::Disk, roots)
    }

    fn reported_through(files: &(impl Files + Sync), roots: &[PathBuf]) -> Reported {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let exit = Cli::from_args(roots.iter().map(OsString::from)).report_to(files, &mut out, &mut err);
        Reported { exit, err: format!("\n{}", String::from_utf8(err).unwrap_or_default()) }
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
        self.exit == ExitCode::SUCCESS && self.err == "\n"
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
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "// note\nmod broken;\n")?;
    Disk::write(root.path(), "src/broken.rs", "\n\nfn (")?;
    let run = Trees::reported(root.path());
    assert_eq!(run.failures(), 1);
    assert!(run.err.contains("cannot parse"));
    assert_eq!(run.violations(), 1);
    Ok(())
}

#[test]
fn a_parse_failure_names_its_line() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "mod broken;\n")?;
    Disk::write(root.path(), "src/broken.rs", "struct A;\nstruct B;\nfn f() { let x = (1; }\n")?;
    assert!(Trees::reported(root.path()).err.contains("src/broken.rs:3"));
    Ok(())
}

#[test]
fn files_outside_crate_targets_and_fixture_trees_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "tests/ui/bad.rs", "fn (")?;
    Disk::write(root.path(), "tooling/corpus/dirty.rs", DIRTY)?;
    assert!(Trees::reported(root.path()).is_clean());
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "tooling/corpus/case/src/lib.rs", DIRTY)?;
    Disk::write(root.path(), "tests/fixtures/bad/src/lib.rs", "fn (")?;
    assert!(Trees::reported(root.path()).is_clean());
    Ok(())
}

#[test]
fn test_modules_in_their_own_files_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "#[cfg(test)]\nmod tests;\n")?;
    Disk::write(root.path(), "src/tests.rs", "struct S { on: bool }\n")?;
    let run = Trees::reported(root.path());
    assert_eq!(run.violations(), 1);
    assert!(!run.err.contains("src/tests.rs"));
    Ok(())
}

#[test]
fn a_declared_module_with_no_file_is_a_failure() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "mod absent;\n")?;
    let run = Trees::reported(root.path());
    assert_eq!(run.failures(), 1);
    assert!(run.err.contains("cannot read"));
    Ok(())
}

#[test]
fn a_tree_with_no_rust_sources_fails_closed() -> Result<(), Box<dyn Error>> {
    let empty = Disk::temporary()?;
    assert!(Trees::reported(empty.path()).err.contains("no Rust sources found"));
    let other = Disk::temporary()?;
    Disk::write(other.path(), "README", "x")?;
    assert!(Trees::reported(other.path()).err.contains("no Rust sources found"));
    Ok(())
}

#[test]
fn every_path_given_is_checked() -> Result<(), Box<dyn Error>> {
    let clean = Disk::package()?;
    Disk::write(clean.path(), "src/lib.rs", CLEAN)?;
    let dirty = Disk::package()?;
    Disk::write(dirty.path(), "src/lib.rs", DIRTY)?;
    let roots = [clean.path(), dirty.path()].map(Path::to_path_buf);
    assert_eq!(Trees::reported_all(&roots).exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn the_binary_root_is_found_whichever_path_is_given() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/main.rs", "struct Extra;\nfn main() {}\n")?;
    assert_eq!(Trees::reported(root.path()).violations(), 1);
    assert_eq!(Trees::reported(&root.path().join("src")).violations(), 1);
    Ok(())
}

#[test]
fn bin_roots_and_modules_named_like_build_directories_are_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/bin/tool.rs", "struct Extra;\nfn main() {}\n")?;
    Disk::write(root.path(), "src/bin/other/main.rs", "mod m {}\nfn main() {}\n")?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "pub mod target;\n")?;
    Disk::write(root.path(), "src/target/mod.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn symlink_loops_do_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", DIRTY)?;
    Disk::linked(root.path(), &root.path().join("src/loop"))?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn a_shebang_is_not_a_comment() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "#!//usr/bin/env foo\nstruct A;\n")?;
    assert!(Trees::reported(root.path()).is_clean());
    Ok(())
}

#[test]
fn a_directory_named_tests_inside_src_is_not_a_package() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "pub mod request;\n")?;
    Disk::write(root.path(), "src/request.rs", "#[cfg(test)]\nmod tests;\n")?;
    Disk::write(root.path(), "src/request/tests.rs", "mod a;\n")?;
    Disk::write(root.path(), "src/request/tests/a.rs", "mod b;\n")?;
    Disk::write(root.path(), "src/request/tests/a/b.rs", "")?;
    assert_eq!(Trees::reported(root.path()).violations(), 1);
    Ok(())
}

#[test]
fn hidden_directories_and_unlisted_crates_are_not_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\nexclude = [\"crates/skipped\"]\n")?;
    Disk::write(root.path(), "crates/one/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "crates/one/src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "crates/skipped/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "crates/skipped/src/lib.rs", DIRTY)?;
    Disk::write(root.path(), "vendor/dep/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "vendor/dep/src/lib.rs", DIRTY)?;
    Disk::write(root.path(), ".claude/worktrees/t1/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), ".claude/worktrees/t1/src/lib.rs", DIRTY)?;
    assert!(Trees::reported(root.path()).is_clean());
    Disk::write(root.path(), "crates/two/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "crates/two/src/lib.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn a_path_dependency_inside_the_workspace_is_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = [\"app\"]\n")?;
    Disk::write(
        root.path(),
        "app/Cargo.toml",
        "[package]\nname = \"app\"\n[dependencies]\nhelp = { path = \"../help\" }\n",
    )?;
    Disk::write(root.path(), "app/src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "help/Cargo.toml", "[package]\nname = \"help\"\n")?;
    Disk::write(root.path(), "help/src/lib.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn the_verdict_does_not_depend_on_where_the_checkout_lives() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    let flow = "#[test]\nfn t() { for i in [2, 3] { if i > 9 { return; } } }\n";
    let build = "fn main() { println!(\"{}\", 2 + 3); }\n";
    let nested = "pub mod a;\npub mod b;\n";
    let layouts = ["plain/proj", "src/proj", "tests/proj"];
    layouts.iter().try_for_each(|layout| {
        let at = root.path().join(layout);
        Disk::write(&at, "Cargo.toml", MANIFEST)?;
        Disk::write(&at, "src/lib.rs", nested)?;
        Disk::write(&at, "src/a.rs", "pub struct Tool;\n")?;
        Disk::write(&at, "src/b.rs", "use crate::a::Tool;\npub struct User(Tool);\n")?;
        Disk::write(&at, "tests/flow.rs", flow)?;
        Disk::write(&at, "build.rs", build)
    })?;
    let counts: Vec<usize> =
        layouts.iter().map(|layout| Trees::reported(&root.path().join(layout)).violations()).collect();
    let first = counts.first().copied().unwrap_or_default();
    assert!(first > 0);
    assert!(counts.iter().all(|count| *count == first));
    Ok(())
}

#[test]
fn a_module_used_by_a_sibling_and_its_test_file_stays_put() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "pub mod a;\n")?;
    Disk::write(root.path(), "src/a.rs", "mod helper;\npub mod user;\n#[cfg(test)]\nmod tests;\n")?;
    Disk::write(root.path(), "src/a/helper.rs", "pub struct Help;\nimpl Help {\n    pub fn lift(&self) {}\n}\n")?;
    Disk::write(root.path(), "src/a/user.rs", "use super::helper::Help;\npub struct User(Help);\n")?;
    Disk::write(root.path(), "src/a/tests.rs", "use super::helper::Help;\n")?;
    assert_eq!(Trees::reported(root.path()).violations(), 1);
    Disk::write(root.path(), "src/a/tests.rs", "")?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn a_crate_is_named_by_its_manifest_not_its_directory() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]\n")?;
    Disk::write(root.path(), "crates/engine-impl/Cargo.toml", "[package]\nname = \"engine\"\n")?;
    Disk::write(root.path(), "crates/engine-impl/src/lib.rs", "mod x;\npub use x::{Used, Unused};\n")?;
    Disk::write(
        root.path(),
        "crates/engine-impl/src/x.rs",
        "pub struct Used;\nimpl Used {\n    pub fn lift(&self) {}\n}\npub struct Unused;\nimpl Unused {\n    pub fn lift(&self) {}\n}\n",
    )?;
    Disk::write(root.path(), "crates/app/Cargo.toml", "[package]\nname = \"app\"\n")?;
    Disk::write(root.path(), "crates/app/src/lib.rs", "pub use engine::Used;\n")?;
    assert_eq!(Trees::reported(root.path()).violations(), 1);
    Ok(())
}

#[test]
fn a_library_and_its_binary_are_judged_apart() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", &Trees::chain("A", 5))?;
    let main = format!("use r::A0;\n{}fn main() {{\n    let _ = (B0, A0);\n}}\n", Trees::chain("B", 4));
    Disk::write(root.path(), "src/main.rs", &main)?;
    assert_eq!(Trees::crate_graph_lines(root.path())?, Lines::default());
    Ok(())
}

#[test]
fn a_library_outside_src_is_judged_and_named_by_its_root_file() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", &format!("{MANIFEST}[lib]\npath = \"code/lib.rs\"\n"))?;
    Disk::write(root.path(), "code/lib.rs", &format!("pub mod a;\n{}", Trees::chain("A", 5)))?;
    Disk::write(root.path(), "code/a.rs", &Trees::chain("B", 5))?;
    let named = format!("{}:1", root.path().join("code/lib.rs").display());
    assert_eq!(Trees::crate_graph_lines(root.path())?, [named.as_str()]);
    Ok(())
}

#[test]
fn inline_modules_and_path_attributes_lead_to_the_file_read() -> Result<(), Box<dyn Error>> {
    let layouts = [
        ("mod outer {\n    mod inner;\n}\n", "src/outer/inner.rs"),
        ("#[path = \"elsewhere/place.rs\"]\nmod renamed;\n", "src/elsewhere/place.rs"),
    ];
    layouts.iter().try_for_each(|(library, module)| {
        let root = Disk::package()?;
        Disk::write(root.path(), "src/lib.rs", library)?;
        Disk::write(root.path(), module, DIRTY)?;
        assert_eq!(Trees::reported(root.path()).violations(), 2);
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
        let root = Disk::temporary()?;
        Disk::write(root.path(), "Cargo.toml", &format!("{MANIFEST}{table}"))?;
        Disk::write(root.path(), file, DIRTY)?;
        assert_eq!(Trees::reported(root.path()).violations(), 2, "{file}");
        Ok(())
    })
}

#[test]
fn an_example_that_is_not_run_as_a_test_is_not_a_skipped_test() -> Result<(), Box<dyn Error>> {
    let declared = [("example", "examples/e.rs", true), ("test", "tests/e.rs", false)];
    declared.iter().try_for_each(|(kind, file, clean)| {
        let root = Disk::temporary()?;
        let table = format!("{MANIFEST}[[{kind}]]\nname = \"e\"\ntest = false\n");
        Disk::write(root.path(), "Cargo.toml", &table)?;
        Disk::write(root.path(), file, CLEAN)?;
        assert_eq!(Trees::reported(root.path()).is_clean(), *clean, "{kind}");
        Ok(())
    })
}

#[test]
fn conditional_paths_and_nested_packages_are_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", "#[cfg_attr(unix, path = \"b.rs\")]\nmod a;\n")?;
    Disk::write(root.path(), "src/a.rs", CLEAN)?;
    Disk::write(root.path(), "src/b.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "inner/Cargo.toml", "[package]\nname = \"inner\"\n")?;
    Disk::write(root.path(), "inner/src/lib.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn a_package_with_its_own_workspace_inside_another_workspace_is_read() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = []\n")?;
    Disk::write(root.path(), "nested/Cargo.toml", "[package]\nname = \"nested\"\n[workspace]\n")?;
    Disk::write(root.path(), "nested/src/lib.rs", DIRTY)?;
    assert_eq!(Trees::reported(root.path()).violations(), 2);
    Ok(())
}

#[test]
fn hidden_and_build_directories_hide_their_packages() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", CLEAN)?;
    ["target/package/copy", ".cache/copy"].iter().try_for_each(|copy| {
        Disk::write(root.path(), &format!("{copy}/Cargo.toml"), "[package]\nname = \"copy\"\n[workspace]\n")?;
        Disk::write(root.path(), &format!("{copy}/src/lib.rs"), DIRTY)
    })?;
    assert!(Trees::reported(root.path()).is_clean());
    Ok(())
}

#[test]
fn a_workspace_manifest_is_not_a_package() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    Disk::write(root.path(), "src/lib.rs", DIRTY)?;
    Disk::write(root.path(), "one/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "one/src/lib.rs", CLEAN)?;
    assert!(Trees::reported(root.path()).is_clean());
    Ok(())
}

#[test]
fn a_manifest_that_cannot_be_read_fails_closed() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::directory(&root.path().join("Cargo.toml"))?;
    assert!(Trees::reported(root.path()).err.contains("Cargo.toml"));
    Ok(())
}

#[test]
fn a_manifest_beside_the_linted_member_that_cannot_be_read_fails_closed() -> Result<(), Box<dyn Error>> {
    let root = Disk::temporary()?;
    Disk::write(root.path(), "Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    Disk::write(root.path(), "one/Cargo.toml", MANIFEST)?;
    Disk::write(root.path(), "one/src/lib.rs", CLEAN)?;
    Disk::write(root.path(), "two/src/lib.rs", CLEAN)?;
    assert!(Trees::reported(&root.path().join("one")).is_clean());
    Disk::directory(&root.path().join("two/Cargo.toml"))?;
    assert!(Trees::reported(&root.path().join("one")).err.contains("two/Cargo.toml"));
    Ok(())
}

#[test]
fn a_directory_that_cannot_be_listed_fails_closed_wherever_it_is_reached() -> Result<(), Box<dyn Error>> {
    let workspace = Disk::temporary()?;
    Disk::write(workspace.path(), "Cargo.toml", "[workspace]\nmembers = [\"one\"]\n")?;
    Disk::write(workspace.path(), "one/Cargo.toml", MANIFEST)?;
    Disk::write(workspace.path(), "one/src/lib.rs", CLEAN)?;
    let package = Disk::package()?;
    Disk::write(package.path(), "src/lib.rs", CLEAN)?;
    let cases = [
        (workspace.path(), "one/src/inner", "one"),
        (workspace.path(), "one/.hidden", "one"),
        (workspace.path(), "two", "one"),
        (package.path(), "tests", "src"),
    ];
    cases.iter().try_for_each(|(root, directory, given)| {
        Disk::write(root, &format!("{directory}/kept.txt"), "x")?;
        let unlistable = file_ports::Disk.canonical(&root.join(directory))?;
        assert!(Trees::reported(&root.join(given)).is_clean(), "{directory}");
        let run = Trees::reported_through(&FakeUnlistable(unlistable.clone()), &[root.join(given)]);
        assert_eq!(run.exit, ExitCode::FAILURE, "{directory}");
        assert!(run.err.contains(&format!("cannot read {}", unlistable.display())), "{directory}: {}", run.err);
        Ok(())
    })
}

#[test]
fn a_twenty_thousand_link_method_chain_is_linted_without_overflowing_the_stack() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    Disk::write(root.path(), "src/lib.rs", &format!("fn long(x: u8) {{ x{} }}\n", ".m()".repeat(LONG_CHAIN_LINKS)))?;
    let run = Trees::reported(root.path());
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
        let root = Disk::package()?;
        Disk::write(root.path(), "src/lib.rs", shape)?;
        let run = Trees::reported(root.path());
        assert_eq!(run.failures(), 1);
        assert!(run.err.contains("src/lib.rs:1"));
        assert!(run.err.contains("nested more than"));
        Ok(())
    })
}

#[test]
fn deep_but_ordinary_nesting_is_still_linted() -> Result<(), Box<dyn Error>> {
    let root = Disk::package()?;
    let nested = format!(
        "fn render_page() {{ let a = {}1{}; }}\n// note\n",
        "(".repeat(SHALLOW_NESTING),
        ")".repeat(SHALLOW_NESTING)
    );
    Disk::write(root.path(), "src/lib.rs", &nested)?;
    let run = Trees::reported(root.path());
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
        let root = Disk::package()?;
        Disk::write(root.path(), "src/lib.rs", source)?;
        assert_eq!(
            Trees::reported(root.path()).err.matches("nested more than").count(),
            *failures,
            "{}",
            &source[..source.len().min(40)]
        );
        Ok(())
    })
}

struct FakeUnlistable(PathBuf);

impl Files for FakeUnlistable {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        file_ports::Disk.read(path)
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        file_ports::Disk.read_text(path)
    }

    fn write(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        file_ports::Disk.write(path, bytes)
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        file_ports::Disk.append(path, bytes)
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        file_ports::Disk.create_directories(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        file_ports::Disk.copy(from, to)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        if directory == self.0 {
            Err(FileFailure::Failed(io::Error::from(io::ErrorKind::PermissionDenied)))
        } else {
            file_ports::Disk.entries(directory)
        }
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        file_ports::Disk.canonical(path)
    }

    fn kind(&self, path: &Path) -> Kind {
        file_ports::Disk.kind(path)
    }
}

mod ports {
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::MANIFEST;

    pub struct Disk;

    impl Disk {
        pub fn entries(directory: &Path) -> io::Result<Vec<PathBuf>> {
            fs::read_dir(directory)?.map(|entry| Ok(entry?.path())).collect()
        }

        pub fn occurrences(path: &Path, needle: &str) -> Result<usize, Box<dyn Error>> {
            if !path.is_dir() {
                return Ok(fs::read_to_string(path).map_or(0, |text| text.matches(needle).count()));
            }
            let mut found = 0;
            for entry in Self::entries(path)? {
                found += Self::occurrences(&entry, needle)?;
            }
            Ok(found)
        }

        pub fn temporary() -> io::Result<tempfile::TempDir> {
            tempfile::tempdir()
        }

        pub fn package() -> Result<tempfile::TempDir, Box<dyn Error>> {
            let root = Self::temporary()?;
            Self::write(root.path(), "Cargo.toml", MANIFEST)?;
            Ok(root)
        }

        pub fn write(root: &Path, path: &str, text: &str) -> Result<(), Box<dyn Error>> {
            let file = root.join(path);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file, text)?;
            Ok(())
        }

        pub fn directory(path: &Path) -> io::Result<()> {
            fs::create_dir(path)
        }

        pub fn linked(target: &Path, link: &Path) -> io::Result<()> {
            std::os::unix::fs::symlink(target, link)
        }
    }
}
