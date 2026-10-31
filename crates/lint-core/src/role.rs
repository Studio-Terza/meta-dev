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
        if inside == Path::new(Self::MAIN)
            || inside.parent() == Some(Path::new(Self::BINARIES))
            || (inside.file_name() == Some(OsStr::new(Self::MAIN))
                && inside.parent().and_then(Path::parent) == Some(Path::new(Self::BINARIES)))
        {
            Role::BinaryRoot
        } else if inside == Path::new(Self::LIBRARY) {
            Role::LibraryRoot
        } else {
            Role::Other
        }
    }
}
