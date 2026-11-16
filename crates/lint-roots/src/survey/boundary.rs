use std::borrow::Borrow;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Spot(Vec<u8>);

impl Spot {
    pub(super) fn bytes(path: &Path) -> &[u8] {
        path.as_os_str().as_encoded_bytes()
    }
}

impl From<PathBuf> for Spot {
    fn from(path: PathBuf) -> Spot {
        Spot(path.into_os_string().into_encoded_bytes())
    }
}

impl Borrow<[u8]> for Spot {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
