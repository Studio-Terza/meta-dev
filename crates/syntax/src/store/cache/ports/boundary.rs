use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Disk;

impl Disk {
    pub const HERE: &'static str = ".";

    pub fn here() -> io::Result<PathBuf> {
        fs::canonicalize(Disk::HERE)
    }

    pub fn text(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    #[must_use]
    pub fn held(path: PathBuf) -> Option<PathBuf> {
        Some(path).filter(|candidate| candidate.is_file())
    }
}
