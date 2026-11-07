use std::fs::FileType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    File,
    Directory,
    Link,
    Missing,
}

impl From<FileType> for Kind {
    fn from(found: FileType) -> Self {
        Some(found)
            .filter(FileType::is_dir)
            .map(|_| Self::Directory)
            .or_else(|| Some(found).filter(FileType::is_symlink).map(|_| Self::Link))
            .unwrap_or(Self::File)
    }
}
