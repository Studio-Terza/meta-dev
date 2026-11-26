use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use test_failure::Failed;

pub struct Design;

impl Design {
    pub fn run(home: &Path, arguments: &[&str]) -> Result<Output, Failed> {
        Design::fed(home, arguments, "")
    }

    pub fn fed(home: &Path, arguments: &[&str], input: &str) -> Result<Output, Failed> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_design"))
            .env("HOME", home)
            .current_dir(home)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child.stdin.take().ok_or("the child's stdin is not piped")?.write_all(input.as_bytes())?;
        Ok(child.wait_with_output()?)
    }
}
