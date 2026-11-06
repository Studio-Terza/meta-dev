use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, FileFailure, Files, Kind, Memory};
use lint_runner::Cli;
use text_lines::Lines;

use crate::fakes::FakeLinked;

const MANIFEST: &str = "[package]\nname = \"r\"\n";
const CLEAN: &str = "struct A;\n";
const HEADLINE: &str = "logic is Rust; web assets live under a directory named assets:\n";
const FINDING: &str = " source outside assets; ";
const REWRITE: &str = "rewrite it in Rust";
const MOVE: &str = "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust";

#[derive(Debug, PartialEq)]
struct Run {
    exit: ExitCode,
    err: String,
}

impl Run {
    fn finding(path: &str, language: &str, fix: &str) -> String {
        format!("{path}:1: {language}{FINDING}{fix}")
    }

    fn lines_at(&self, path: &str) -> Vec<String> {
        let site = format!("{path}:1: ");
        Lines::from(self.err.as_str()).filter(|line| line.contains(&site)).map(str::to_owned).collect()
    }

    fn findings(&self) -> usize {
        self.err.matches(FINDING).count()
    }
}

struct Checkout {
    files: Memory,
    root: PathBuf,
}

impl Checkout {
    const ROOT: &'static str = "/checkout";

    fn new() -> Result<Checkout, FileFailure> {
        let checkout = Checkout { files: Memory::default(), root: PathBuf::from(Self::ROOT) };
        checkout.write("Cargo.toml", MANIFEST)?;
        checkout.write("src/lib.rs", CLEAN)?;
        Ok(checkout)
    }

    fn below(self, directory: &str) -> Checkout {
        Checkout { root: self.root.join(directory), files: self.files }
    }

    fn write(&self, path: &str, text: &str) -> Result<PathBuf, FileFailure> {
        let file = self.root.join(path);
        file.parent().map_or(Ok(()), |parent| self.files.create_directories(parent))?;
        self.files.write(&file, Bytes::from(text))?;
        Ok(file)
    }

    fn run(files: &(impl Files + Sync), roots: &[PathBuf]) -> Run {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(roots.iter().map(OsString::from)).report_to(files, &mut out, &mut err);
        Run { exit, err: String::from_utf8(err).unwrap_or_default() }
    }

    fn checked(&self) -> Run {
        Self::run(&self.files, std::slice::from_ref(&self.root))
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn shown(&self, path: &str) -> String {
        format!("{}/{path}", self.path().display())
    }
}

#[test]
fn a_shell_script_fails_the_run_and_names_the_fix() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "echo hi\n")?;
    let run = checkout.checked();
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains(HEADLINE));
    assert_eq!(
        run.lines_at(&checkout.shown("run.sh")),
        vec![Run::finding(&checkout.shown("run.sh"), "shell", REWRITE)]
    );
    assert_eq!(run.findings(), 1);
    Ok(())
}

#[test]
fn a_tree_of_rust_and_configuration_passes() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/workflows/ci.yml", "name: ci\n")?;
    checkout.write("Dockerfile", "FROM scratch\n")?;
    assert_eq!(checkout.checked(), Run { exit: ExitCode::SUCCESS, err: String::new() });
    Ok(())
}

#[test]
fn a_workflow_run_step_with_shell_logic_fails_the_run() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/workflows/ci.yml", "jobs:\n  a:\n    steps:\n      - run: cargo test | tee out\n")?;
    let run = checkout.checked();
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains("a workflow run step is a short command; logic lives in Rust:\n"));
    assert!(
        run.err.contains(&format!(
            "{}:4: a run step uses the shell operator |",
            checkout.shown(".github/workflows/ci.yml")
        ))
    );
    Ok(())
}

#[test]
fn hidden_directories_are_read() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/scripts/matrix.py", "print(1)\n")?;
    let run = checkout.checked();
    assert_eq!(
        run.lines_at(&checkout.shown(".github/scripts/matrix.py")),
        vec![Run::finding(&checkout.shown(".github/scripts/matrix.py"), "Python", REWRITE)]
    );
    assert_eq!(run.findings(), 1);
    Ok(())
}

#[test]
fn version_control_and_build_output_are_not_read() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".git/hooks/pre.sh", "x\n")?;
    checkout.write("target/debug/build.sh", "x\n")?;
    assert_eq!(checkout.checked().findings(), 0);
    Ok(())
}

#[test]
fn a_directory_named_target_without_a_manifest_beside_it_is_read() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("docs/target/run.sh", "x\n")?;
    assert_eq!(
        checkout.checked().lines_at(&checkout.shown("docs/target/run.sh")),
        vec![Run::finding(&checkout.shown("docs/target/run.sh"), "shell", REWRITE)]
    );
    Ok(())
}

#[test]
fn files_below_assets_are_web_assets_and_pass() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    let written: Vec<PathBuf> = ["site/assets/app.js", "assets/css/site.css", "site/app.js"]
        .iter()
        .map(|path| checkout.write(path, "x\n"))
        .collect::<Result<Vec<PathBuf>, FileFailure>>()?;
    let kinds: Vec<Kind> = written.iter().map(|file| checkout.files.kind(file)).collect();
    assert_eq!(kinds, vec![Kind::File; 3]);
    let run = checkout.checked();
    assert_eq!(
        run.lines_at(&checkout.shown("site/app.js")),
        vec![Run::finding(&checkout.shown("site/app.js"), "JavaScript", MOVE)]
    );
    assert_eq!(run.findings(), 1);
    Ok(())
}

#[test]
fn a_checkout_below_a_directory_named_assets_is_judged_by_its_own_paths() -> Result<(), Box<dyn Error>> {
    let nested = Checkout::new()?.below("inner/assets/nested");
    nested.write("Cargo.toml", MANIFEST)?;
    nested.write("src/lib.rs", CLEAN)?;
    nested.write("run.sh", "x\n")?;
    let run = nested.checked();
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert_eq!(run.lines_at(&nested.shown("run.sh")), vec![Run::finding(&nested.shown("run.sh"), "shell", REWRITE)]);
    Ok(())
}

#[test]
fn a_path_given_twice_is_reported_once() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    let roots = [checkout.path().to_path_buf(), checkout.path().to_path_buf()];
    assert_eq!(Checkout::run(&checkout.files, &roots).findings(), 1);
    Ok(())
}

#[test]
fn a_symlink_loop_does_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    let linked = FakeLinked::new(&checkout.files, checkout.path().join("src/loop"), checkout.path().to_path_buf());
    assert_eq!(Checkout::run(&linked, std::slice::from_ref(&checkout.root)).findings(), 1);
    Ok(())
}
