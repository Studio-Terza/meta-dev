use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("{manifest} is not inside the workspace", manifest = manifest.display())]
pub struct Outside {
    manifest: PathBuf,
}

impl Outside {
    pub(crate) const fn new(manifest: PathBuf) -> Self {
        Self { manifest }
    }
}
