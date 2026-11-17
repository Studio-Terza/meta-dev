use newtype::{Arbitrary, Newtype};
use std::path::PathBuf;

#[derive(Newtype, Arbitrary, Debug, thiserror::Error)]
#[error("{manifest} is not inside the workspace", manifest = manifest.display())]
pub struct Outside {
    manifest: PathBuf,
}
