use std::path::PathBuf;

use crate::{Parses, Schema, StoreFiller};

pub trait Opens: Parses {
    type Store: StoreFiller;

    fn file(&self, schema: &Schema, path: PathBuf) -> Self::Store;

    fn in_memory(&self, schema: &Schema) -> Self::Store;
}
