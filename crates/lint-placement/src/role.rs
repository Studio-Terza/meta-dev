use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    BinaryRoot,
    LibraryRoot,
    Target,
    BuildScript,
    Other,
}

impl Role {
    const BINARIES: &'static str = "bin";
    const MAIN: &'static str = "main.rs";
    const LIBRARY: &'static str = "lib.rs";

    #[must_use]
    pub fn roots(self, target: &Path, file: &Path) -> Vec<PathBuf> {
        match self {
            Role::BinaryRoot | Role::BuildScript => vec![file.to_path_buf()],
            Role::Target => vec![target.to_path_buf(), target.join(Self::MAIN)],
            Role::LibraryRoot | Role::Other => Vec::new(),
        }
    }

    #[must_use]
    pub fn in_source(inside: &Path) -> Role {
        Self::binary(inside)
            .or_else(|| {
                Some(inside).filter(|candidate| *candidate == Path::new(Self::LIBRARY)).map(|_| Role::LibraryRoot)
            })
            .unwrap_or(Role::Other)
    }

    fn binary(inside: &Path) -> Option<Role> {
        let main = Some(inside).filter(|candidate| *candidate == Path::new(Self::MAIN));
        let listed = inside.parent().filter(|parent| *parent == Path::new(Self::BINARIES));
        let nested = Some(inside)
            .filter(|candidate| candidate.file_name() == Some(OsStr::new(Self::MAIN)))
            .and_then(|main_file| main_file.parent().and_then(Path::parent))
            .filter(|grandparent| *grandparent == Path::new(Self::BINARIES));
        main.or(listed).or(nested).map(|_| Role::BinaryRoot)
    }
}
