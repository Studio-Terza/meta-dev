use std::env;
use std::ffi::OsString;
use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use file_ports::Bytes;
use tempfile::TempDir;

#[derive(Debug)]
pub struct Home {
    directory: TempDir,
}

impl Home {
    const RUNNABLE: u32 = 0o755;

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
        fs::write(&placed, contents)?;
        fs::set_permissions(placed, Permissions::from_mode(Self::RUNNABLE))?;
        Ok(self)
    }

    pub fn run(&self, program: &Path, arguments: &[OsString]) -> io::Result<Output> {
        let inherited = env::var_os("PATH").unwrap_or_default();
        let searched = [self.directory.path().join("bin")].into_iter().chain(env::split_paths(&inherited));
        let path = env::join_paths(searched).map_err(io::Error::other)?;
        Command::new(program).env("HOME", self.directory.path()).env("PATH", path).args(arguments).output()
    }
}
