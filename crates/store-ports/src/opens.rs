use std::path::PathBuf;

use crate::{Schema, StoreFiller};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Fresh,
    Copied,
}

pub trait Opens {
    type Store: StoreFiller;

    fn file<S: Schema>(&self, path: PathBuf) -> Self::Store;

    fn in_memory<S: Schema>(&self) -> Self::Store;
}
