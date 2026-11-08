use std::path::Path;

use file_ports::Bytes;

use crate::CargoFailure;

pub trait Cargo {
    fn metadata(&self, manifest: &Path) -> Result<Bytes, CargoFailure>;

    fn metadata_many(&self, manifests: &[impl AsRef<Path>]) -> Vec<Result<Bytes, CargoFailure>> {
        manifests.iter().map(|manifest| self.metadata(manifest.as_ref())).collect()
    }
}
