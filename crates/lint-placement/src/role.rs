use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    BinaryRoot,
    LibraryRoot,
    Target,
    Other,
}

impl Role {
    const BINARIES: &'static str = "bin";
    const MAIN: &'static str = "main.rs";
    const LIBRARY: &'static str = "lib.rs";

    #[must_use]
    pub fn in_source(inside: &Path) -> Role {
        Self::binary(inside)
            .or_else(|| Some(inside).filter(|inside| *inside == Path::new(Self::LIBRARY)).map(|_| Role::LibraryRoot))
            .unwrap_or(Role::Other)
    }

    fn binary(inside: &Path) -> Option<Role> {
        let main = Some(inside).filter(|inside| *inside == Path::new(Self::MAIN));
        let listed = inside.parent().filter(|parent| *parent == Path::new(Self::BINARIES));
        let nested = Some(inside)
            .filter(|inside| inside.file_name() == Some(OsStr::new(Self::MAIN)))
            .and_then(|inside| inside.parent().and_then(Path::parent))
            .filter(|grandparent| *grandparent == Path::new(Self::BINARIES));
        main.or(listed).or(nested).map(|_| Role::BinaryRoot)
    }
}
