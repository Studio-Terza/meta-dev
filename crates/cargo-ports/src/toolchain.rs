use std::path::Path;
use std::process::{Command, Output, Stdio};

use file_ports::{Bytes, Text};

use crate::{Cargo, CargoFailure};

#[derive(Clone, Copy, Debug, Default)]
pub struct Toolchain;

impl Toolchain {
    fn answered(manifest: &Path, output: Output) -> Result<Bytes, CargoFailure> {
        let Output { status, stdout, stderr } = output;
        status.success().then(|| Bytes::new(stdout)).ok_or_else(|| CargoFailure::Exited {
            manifest: manifest.to_path_buf(),
            status,
            stderr: Text::new(String::from_utf8(stderr).unwrap_or_default()),
        })
    }
}

impl Cargo for Toolchain {
    fn metadata(&self, manifest: &Path) -> Result<Bytes, CargoFailure> {
        Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--locked", "--offline", "--manifest-path"])
            .arg(manifest)
            .stdin(Stdio::null())
            .output()
            .map_err(|source| CargoFailure::Unstarted { manifest: manifest.to_path_buf(), source })
            .and_then(|output| Self::answered(manifest, output))
    }
}
