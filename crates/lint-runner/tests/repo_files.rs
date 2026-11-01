use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lint_runner::Cli;

const MANIFEST: &str = "[package]\nname = \"r\"\n";
const CLEAN: &str = "struct A;\n";
const SHELL: &str = "run.sh:1: shell source outside assets; rewrite it in Rust";

#[derive(Debug, PartialEq)]
struct Run {
    exit: ExitCode,
    err: String,
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

    fn findings(&self) -> Vec<String> {
        let root = format!("{}/", self.path().display());
        Self::run(&[self.path().to_path_buf()])
            .err
            .lines()
            .filter(|line| line.contains(" source outside assets"))
            .map(|line| line.replace(&root, ""))
            .collect()
    }
}

#[test]
fn a_shell_script_fails_the_run_and_names_the_fix() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "echo hi\n")?;
    let run = Checkout::run(&[checkout.path().to_path_buf()]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains("logic is Rust; web assets live under a directory named assets:\n"));
    assert_eq!(checkout.findings(), [SHELL]);
    Ok(())
}

#[test]
fn a_tree_of_rust_and_configuration_passes() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/workflows/ci.yml", "name: ci\n")?;
    checkout.write("Dockerfile", "FROM scratch\n")?;
    assert_eq!(Checkout::run(&[checkout.path().to_path_buf()]), Run { exit: ExitCode::SUCCESS, err: String::new() });
    Ok(())
}

#[test]
fn hidden_directories_are_read_but_version_control_and_build_output_are_not() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write(".github/scripts/matrix.py", "print(1)\n")?;
    checkout.write(".git/hooks/pre.sh", "x\n")?;
    checkout.write("target/debug/build.sh", "x\n")?;
    assert_eq!(checkout.findings(), [".github/scripts/matrix.py:1: Python source outside assets; rewrite it in Rust"]);
    Ok(())
}

#[test]
fn a_directory_named_target_without_a_manifest_beside_it_is_read() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("docs/target/run.sh", "x\n")?;
    assert_eq!(checkout.findings(), ["docs/target/run.sh:1: shell source outside assets; rewrite it in Rust"]);
    Ok(())
}

#[test]
fn files_below_assets_are_web_assets_and_pass() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    ["site/assets/app.js", "assets/css/site.css", "site/app.js"]
        .iter()
        .try_for_each(|path| checkout.write(path, "x\n"))?;
    let findings = checkout.findings();
    assert_eq!(findings.len(), 1);
    assert!(findings.iter().all(|line| line.starts_with("site/app.js:1: JavaScript source outside assets; ")));
    Ok(())
}

#[test]
fn a_checkout_below_a_directory_named_assets_is_judged_by_its_own_paths() -> Result<(), Box<dyn Error>> {
    let outer = Checkout::new()?;
    let inner = outer.path().join("assets/repo");
    fs::create_dir_all(inner.join("src"))?;
    fs::write(inner.join("Cargo.toml"), MANIFEST)?;
    fs::write(inner.join("src/lib.rs"), CLEAN)?;
    fs::write(inner.join("run.sh"), "x\n")?;
    let run = Checkout::run(&[inner.clone()]);
    assert_eq!(run.exit, ExitCode::FAILURE);
    assert!(run.err.contains(&format!("{}/run.sh:1: shell source", inner.display())));
    Ok(())
}

#[test]
fn a_path_given_twice_is_reported_once() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    let roots = [checkout.path().to_path_buf(), checkout.path().to_path_buf()];
    assert_eq!(Checkout::run(&roots).err.matches("run.sh:1").count(), 1);
    Ok(())
}

#[test]
fn a_symlink_loop_does_not_repeat_findings() -> Result<(), Box<dyn Error>> {
    let checkout = Checkout::new()?;
    checkout.write("run.sh", "x\n")?;
    std::os::unix::fs::symlink(checkout.path(), checkout.path().join("src/loop"))?;
    assert_eq!(checkout.findings().len(), 1);
    Ok(())
}
