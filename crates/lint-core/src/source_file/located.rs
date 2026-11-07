use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Located {
    Shown,
    At(PathBuf),
}

impl Located {
    #[must_use]
    pub fn within<'path>(&'path self, shown: &'path Path) -> &'path Path {
        match self {
            Self::Shown => shown,
            Self::At(located) => located,
        }
    }
}
