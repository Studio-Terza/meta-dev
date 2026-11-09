use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Disk;

impl Disk {
    const MANIFEST: &'static str = "CARGO_MANIFEST_DIR";

    pub fn manifest() -> PathBuf {
        env::var_os(Self::MANIFEST).map(PathBuf::from).unwrap_or_default()
    }

    pub fn text(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    pub fn held(path: PathBuf) -> Option<PathBuf> {
        Some(path).filter(|candidate| candidate.is_file())
    }

    pub fn listed(directory: &Path) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(directory)?.map(|entry| entry.map(|found| found.path())).collect()
    }
}
