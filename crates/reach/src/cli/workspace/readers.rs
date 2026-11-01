use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use newtype::Newtype;

use crate::cli::workspace::crate_name::CrateName;

#[derive(Newtype, Debug)]
pub struct Readers(BTreeMap<CrateName, Vec<PathBuf>>);

impl Readers {
    pub fn reached_by(&self, path: &Path) -> impl Iterator<Item = CrateName> {
        self.0.iter().filter(|(_, reads)| reads.iter().any(|read| path.starts_with(read))).map(|(name, _)| name.clone())
    }
}
