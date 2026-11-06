use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

use file_ports::Bytes;
use tempfile::TempDir;

#[derive(Debug)]
pub struct Home {
    directory: TempDir,
}

impl Home {
    pub fn holding(file: &Path, contents: &Bytes) -> io::Result<Home> {
        let directory = TempDir::new()?;
        let placed = directory.path().join(file);
        fs::create_dir_all(placed.parent().unwrap_or(directory.path()))?;
        fs::write(placed, contents)?;
        Ok(Home { directory })
    }

    pub fn run(&self, program: &Path, arguments: &[OsString]) -> io::Result<Output> {
        Command::new(program).env("HOME", self.directory.path()).args(arguments).output()
    }
}
