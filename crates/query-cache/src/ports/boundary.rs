use std::env;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Manifest;

impl Manifest {
    const DIRECTORY: &'static str = "CARGO_MANIFEST_DIR";

    pub(crate) fn directory() -> PathBuf {
        env::var_os(Self::DIRECTORY).map_or_default(PathBuf::from)
    }
}
