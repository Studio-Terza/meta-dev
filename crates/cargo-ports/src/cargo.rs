use std::path::Path;

use file_ports::Bytes;

pub trait Cargo {
    fn metadata(&self, manifest: &Path) -> Option<Bytes>;

    fn metadata_many(&self, manifests: &[impl AsRef<Path>]) -> Vec<Option<Bytes>> {
        manifests.iter().map(|manifest| self.metadata(manifest.as_ref())).collect()
    }
}
