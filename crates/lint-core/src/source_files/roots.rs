mod directory;
mod package;

use std::collections::BTreeSet;
use std::path::PathBuf;

use directory::Directory;
use package::Package;

use super::tree::Tree;
use crate::failure::Failure;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    pub fn under(tree: &Tree) -> Result<BTreeSet<PathBuf>, Failure> {
        let below = Directory::new(tree.canonical().to_path_buf()).tree()?;
        let above = tree.canonical().ancestors().skip(1).map(|ancestor| Directory::new(ancestor.to_path_buf()));
        below.into_iter().chain(above).map(Package::in_directory).try_fold(BTreeSet::new(), |roots, package| {
            Ok(roots
                .into_iter()
                .chain(package.targets()?.into_iter().filter(|file| file.starts_with(tree.canonical())))
                .collect())
        })
    }
}
