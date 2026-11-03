use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use newtype::Newtype;

use crate::cli::workspace::crate_name::CrateName;

#[derive(Newtype, Debug)]
pub struct Readers<'meta>(BTreeMap<&'meta CrateName, &'meta [PathBuf]>);

impl<'meta> Readers<'meta> {
    pub fn reached_by(&self, path: &Path) -> impl Iterator<Item = &'meta CrateName> {
        self.0.iter().filter(|(_, reads)| reads.iter().any(|read| path.starts_with(read))).map(|(name, _)| *name)
    }
}
