use std::path::PathBuf;

use cargo_ports::CargoFailure;
use file_ports::FileFailure;

#[derive(Debug, thiserror::Error)]
pub enum Unlocated {
    #[error("cannot describe the workspace: {source}")]
    Metadata {
        #[from]
        source: CargoFailure,
    },
    #[error("cannot parse cargo metadata for {manifest}: {source}", manifest = manifest.display())]
    Unparsed { manifest: PathBuf, source: serde_json::Error },
    #[error("cannot read the crate root {root}: {source}", root = root.display())]
    Root { root: PathBuf, source: FileFailure },
}
