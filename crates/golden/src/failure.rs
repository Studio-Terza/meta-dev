use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::{Difference, Mode};

#[derive(Debug, Error)]
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
    #[error(
        "{}: a test cannot choose to record a package's golden; run `{}={} cargo test` to record it",
        path.display(), Mode::VARIABLE, Mode::ON
    )]
    NotChosen { path: PathBuf },
    #[error("{}: a golden name stays inside tests/goldens; no `..`, root or drive", name.display())]
    Escapes { name: PathBuf },
}
