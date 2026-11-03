use std::path::Path;

use file_ports::{Files, Kind};

#[derive(Debug)]
pub struct Followed;

impl Followed {
    #[must_use]
    pub fn file<'path>(files: &impl Files, path: &'path Path) -> Option<&'path Path> {
        files.canonical(path).ok().filter(|real| files.kind(real) == Kind::File).map(|_| path)
    }
}
