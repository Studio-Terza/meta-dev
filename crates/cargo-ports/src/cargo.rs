use std::path::PathBuf;

use file_ports::Bytes;
use port_core::port;

use crate::CargoFailure;

#[port]
pub trait Cargo {
    fn metadata(&self, manifests: &[PathBuf]) -> Vec<Result<Bytes, CargoFailure>>;
}
