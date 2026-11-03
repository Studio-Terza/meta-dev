use std::fs::FileType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    File,
    Directory,
    Link,
    Missing,
}

impl From<FileType> for Kind {
    fn from(found: FileType) -> Kind {
        Some(found)
            .filter(FileType::is_dir)
            .map(|_| Kind::Directory)
            .or_else(|| Some(found).filter(FileType::is_symlink).map(|_| Kind::Link))
            .unwrap_or(Kind::File)
    }
}
