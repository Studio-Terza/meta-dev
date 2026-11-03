use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::input::Input;

#[derive(Debug)]
pub struct Pending {
    bytes: PathBuf,
    role: PathBuf,
}

impl Pending {
    pub fn of(directory: &Path, target: &str) -> Pending {
        Pending {
            bytes: directory.join(format!("{target}-pending.rs")),
            role: directory.join(format!("{target}-pending.role")),
        }
    }

    pub fn record(&self, input: &Input) -> io::Result<()> {
        if let Some(directory) = self.bytes.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(&self.bytes, input.bytes())?;
        fs::write(&self.role, input.name())
    }

    pub fn clear(&self) -> io::Result<()> {
        fs::remove_file(&self.bytes)?;
        fs::remove_file(&self.role)
    }
}
