use std::path::Path;
use std::process::{Command, Stdio};

use file_ports::Bytes;

use crate::Cargo;

#[derive(Clone, Copy, Debug, Default)]
pub struct Toolchain;

impl Cargo for Toolchain {
    fn metadata(&self, manifest: &Path) -> Option<Bytes> {
        Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--locked", "--offline", "--manifest-path"])
            .arg(manifest)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| Bytes::new(output.stdout))
    }
}
