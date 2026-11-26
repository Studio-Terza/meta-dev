use std::error::Error;
use std::path::Path;
use std::process::{Command, Output};

pub struct Design;

impl Design {
    pub fn run(home: &Path, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
        Ok(Command::new(env!("CARGO_BIN_EXE_design")).env("HOME", home).current_dir(home).args(arguments).output()?)
    }
}
