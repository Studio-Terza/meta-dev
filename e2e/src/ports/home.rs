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
    pub fn new() -> io::Result<Home> {
        TempDir::new().map(|directory| Home { directory })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    pub fn holding(self, file: &Path, contents: &Bytes) -> io::Result<Home> {
        let placed = self.directory.path().join(file);
        fs::create_dir_all(placed.parent().unwrap_or(self.directory.path()))?;
        fs::write(placed, contents)?;
        Ok(self)
    }

    pub fn run(&self, program: &Path, arguments: &[OsString]) -> io::Result<Output> {
        Command::new(program).env("HOME", self.directory.path()).args(arguments).output()
    }
}
