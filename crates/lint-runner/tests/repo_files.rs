use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lint_runner::Cli;

const MANIFEST: &str = "[package]\nname = \"r\"\n";
const CLEAN: &str = "struct A;\n";
const HEADLINE: &str = "logic is Rust; web assets live under a directory named assets:\n";
const FINDING: &str = " source outside assets; ";
const REWRITE: &str = "rewrite it in Rust\n";
const MOVE: &str = "if it is a web asset, move it under a directory named assets; otherwise rewrite it in Rust\n";

#[derive(Debug, PartialEq)]
struct Run {
    exit: ExitCode,
    err: String,
}

impl Run {
    fn names(&self, path: &str, language: &str, fix: &str) -> bool {
        self.err.contains(&format!("{path}:1: {language}{FINDING}{fix}"))
    }

    fn findings(&self) -> usize {
        self.err.matches(FINDING).count()
    }
}

struct Checkout(tempfile::TempDir);

impl Checkout {
    fn new() -> Result<Checkout, Box<dyn Error>> {
        let checkout = Checkout(tempfile::tempdir()?);
        checkout.write("Cargo.toml", MANIFEST)?;
        checkout.write("src/lib.rs", CLEAN)?;
        Ok(checkout)
    }

    fn write(&self, path: &str, text: &str) -> Result<(), Box<dyn Error>> {
        let file = self.0.path().join(path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(fs::write(file, text)?)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn run(roots: &[PathBuf]) -> Run {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(roots.iter().map(OsString::from)).report_to(&mut out, &mut err);
        Run { exit, err: String::from_utf8(err).unwrap_or_default() }
    }

    fn checked(&self) -> Run {
        Self::run(&[self.path().to_path_buf()])
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
    assert!(run.names(&checkout.shown("run.sh"), "shell", REWRITE));
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
fn hidden_directories_are_read() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/scripts/matrix.py", "print(1)\n")?;
    let run = checkout.checked();
    assert!(run.names(&checkout.shown(".github/scripts/matrix.py"), "Python", REWRITE));
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
    assert!(checkout.checked().names(&checkout.shown("docs/target/run.sh"), "shell", REWRITE));
    Ok(())
}

#[test]
fn files_below_assets_are_web_assets_and_pass() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    ["site/assets/app.js", "assets/css/site.css", "site/app.js"]
        .iter()
        .try_for_each(|path| checkout.write(path, "x\n"))?;
    let run = checkout.checked();
    assert!(run.names(&checkout.shown("site/app.js"), "JavaScript", MOVE));
    assert_eq!(run.findings(), 1);
    Ok(())
}

#[test]
fn a_checkout_below_a_directory_named_assets_is_judged_by_its_own_paths() -> Result<(), Box<dyn Error>> {
    let outer = Checkout::new()?;
    let inner = Checkout(tempfile::tempdir_in(outer.path())?);
    fs::create_dir(inner.path().join("assets"))?;
    let nested = Checkout(tempfile::tempdir_in(inner.path().join("assets"))?);
    nested.write("Cargo.toml", MANIFEST)?;
    nested.write("src/lib.rs", CLEAN)?;
    nested.write("run.sh", "x\n")?;
    let run = nested.checked();
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.names(&nested.shown("run.sh"), "shell", REWRITE));
    Ok(())
}

#[test]
fn a_path_given_twice_is_reported_once() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    let roots = [checkout.path().to_path_buf(), checkout.path().to_path_buf()];
    assert_eq!(Checkout::run(&roots).findings(), 1);
    Ok(())
}

#[test]
fn a_symlink_loop_does_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    std::os::unix::fs::symlink(checkout.path(), checkout.path().join("src/loop"))?;
    assert_eq!(checkout.checked().findings(), 1);
    Ok(())
}
