use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::Disk;
use crate::failure::Failure;

impl Disk {
    pub(crate) fn read(path: &Path) -> Result<String, Failure> {
        fs::read_to_string(path).map_err(|source| Failure::Unreadable { path: path.to_path_buf(), source })
    }

    pub(crate) fn write(path: &Path, text: &str) -> Result<(), Failure> {
        fs::write(path, text).map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }

    pub(crate) fn append(path: &Path, text: &str) -> Result<(), Failure> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(text.as_bytes()))
            .map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }
}
