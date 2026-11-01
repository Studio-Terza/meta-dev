use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::{Difference, Mode};

#[derive(Error)]
pub enum Failure {
    #[error(
        "{}: no golden; run `{}={} cargo test` to record it, then review it",
        path.display(), Mode::VARIABLE, Mode::ON
    )]
    Missing { path: PathBuf },
    #[error("{}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{}: {source}", path.display())]
    Unwritable { path: PathBuf, source: io::Error },
    #[error(
        "{}: output differs from the golden\n{difference}\nif this change is intended, run `{}={} cargo test` and review the diff",
        path.display(), Mode::VARIABLE, Mode::ON
    )]
    Differs { path: PathBuf, difference: Box<Difference> },
    #[error("{}={value:?} is not a value; {0}={1} records, unset checks", Mode::VARIABLE, Mode::ON)]
    UnknownValue { value: OsString },
    #[error("{} is refused in CI; record locally and commit the diff", Mode::VARIABLE)]
    RecordInCi,
    #[error("{}: a golden name stays inside tests/goldens; no `..`, root or drive", name.display())]
    Escapes { name: PathBuf },
    #[error("CARGO_MANIFEST_DIR is not set; run goldens through `cargo test`")]
    NoPackage,
}

impl fmt::Debug for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
