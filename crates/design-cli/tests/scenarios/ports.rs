use std::path::Path;
use std::process::{Command, Output};
use test_failure::Failed;

pub struct Design;

impl Design {
    pub fn run(home: &Path, arguments: &[&str]) -> Result<Output, Failed> {
        Ok(Command::new(env!("CARGO_BIN_EXE_design")).env("HOME", home).current_dir(home).args(arguments).output()?)
    }
}
