use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};

use file_ports::Text;
use port_core::{Pass, port};

#[port]
pub trait Cargo {
    fn metadata(&self, roots: &[PathBuf]) -> Vec<io::Result<Text>>;
}

#[derive(Clone, Copy, Debug)]
pub struct Toolchain;

impl Toolchain {
    const PROGRAM: &'static str = "cargo";
    const METADATA: [&'static str; 4] = ["metadata", "--format-version", "1", "--locked"];
    const HERE: &'static str = ".";
}

impl CargoFiller for Toolchain {
    fn metadata(&self, _pass: Pass<'_, cargo_ops::Metadata>, roots: &[PathBuf]) -> Vec<io::Result<Text>> {
        roots
            .iter()
            .map(|root| {
                let directory = root.ancestors().find(|ancestor| ancestor.is_dir()).unwrap_or(Path::new(Self::HERE));
                let Output { status, stdout, stderr } =
                    Command::new(Self::PROGRAM).args(Self::METADATA).current_dir(directory).output()?;
                Some(status).filter(ExitStatus::success).map(|_| stdout).ok_or(stderr).map_or_else(
                    |stderr| Err(io::Error::other(String::from_utf8(stderr).unwrap_or_default())),
                    |stdout| String::from_utf8(stdout).map(Text::new).map_err(io::Error::other),
                )
            })
            .collect()
    }
}
