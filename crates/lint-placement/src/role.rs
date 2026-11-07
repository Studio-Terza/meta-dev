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
            Self::BinaryRoot | Self::BuildScript => vec![file.to_path_buf()],
            Self::Target => vec![target.to_path_buf(), target.join(Self::MAIN)],
            Self::LibraryRoot | Self::Other => Vec::new(),
        }
    }

    #[must_use]
    pub fn in_source(inside: &Path) -> Self {
        Self::binary(inside)
            .or_else(|| Some(inside).filter(|inside| *inside == Path::new(Self::LIBRARY)).map(|_| Self::LibraryRoot))
            .unwrap_or(Self::Other)
    }

    fn binary(inside: &Path) -> Option<Self> {
        let main = Some(inside).filter(|inside| *inside == Path::new(Self::MAIN));
        let listed = inside.parent().filter(|parent| *parent == Path::new(Self::BINARIES));
        let nested = Some(inside)
            .filter(|inside| inside.file_name() == Some(OsStr::new(Self::MAIN)))
            .and_then(|inside| inside.parent().and_then(Path::parent))
            .filter(|grandparent| *grandparent == Path::new(Self::BINARIES));
        main.or(listed).or(nested).map(|_| Self::BinaryRoot)
    }
}
