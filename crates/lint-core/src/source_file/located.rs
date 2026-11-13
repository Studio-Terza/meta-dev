use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum Located {
    Shown,
    At(PathBuf),
}

impl Located {
    #[must_use]
    pub fn within<'path>(&'path self, shown: &'path Path) -> &'path Path {
        match self {
            Located::Shown => shown,
            Located::At(located) => located,
        }
    }
}
