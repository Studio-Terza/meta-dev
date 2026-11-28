use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Disk;

impl Disk {
    const MANIFEST: &'static str = "CARGO_MANIFEST_DIR";

    pub(crate) fn manifest() -> PathBuf {
        env::var_os(Self::MANIFEST).map_or_default(PathBuf::from)
    }

    pub(crate) fn text(path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    pub(crate) fn held(path: PathBuf) -> Option<PathBuf> {
        Some(path).filter(|candidate| candidate.is_file())
    }
}
