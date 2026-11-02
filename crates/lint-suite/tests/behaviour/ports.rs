use std::fs;
use std::path::{Path, PathBuf};

pub struct Checkout(tempfile::TempDir);

impl Checkout {
    pub fn new() -> Checkout {
        Checkout(tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory")))
    }

    pub fn with(self, path: &str, text: &str) -> Checkout {
        let file = self.file(path);
        file.parent().map(fs::create_dir_all);
        fs::write(&file, text).ok();
        self
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    pub fn file(&self, path: &str) -> PathBuf {
        self.0.path().join(path)
    }

    pub fn shown(&self) -> String {
        self.0.path().display().to_string()
    }
}
