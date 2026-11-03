use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("{manifest} is not inside the workspace", manifest = manifest.display())]
pub struct Outside {
    manifest: PathBuf,
}

impl Outside {
    pub(crate) fn new(manifest: PathBuf) -> Outside {
        Outside { manifest }
    }
}
