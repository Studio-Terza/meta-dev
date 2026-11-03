mod admission;
mod directory;
mod package;
mod target;

use admission::Admission;
use directory::Directory;
use package::Package;

pub use target::Target;

use std::path::PathBuf;

use crate::tree::Tree;
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    pub fn files(files: &impl Files, tree: &Tree) -> Result<Vec<PathBuf>, Failure> {
        Directory::new(tree.canonical().to_path_buf()).files(files)
    }

    pub fn under(files: &impl Files, tree: &Tree) -> Result<Vec<Target>, Failure> {
        let below = Directory::new(tree.canonical().to_path_buf()).tree(files)?;
        let above = tree.canonical().ancestors().skip(1).map(|ancestor| Directory::new(ancestor.to_path_buf()));
        let manifests = below.into_iter().chain(above).try_fold(Vec::new(), |found, directory| {
            Ok(found
                .into_iter()
                .chain(Manifest::read(files, directory.as_ref())?.filter(|manifest| manifest.package().is_some()))
                .collect())
        })?;
        Admission::members_among(files, manifests)?.iter().try_fold(Vec::new(), |roots, manifest| {
            Ok(roots
                .into_iter()
                .chain(
                    Package::in_directory(manifest.directory().to_path_buf(), manifest.declared_roots(files))
                        .targets_under(files, tree)?,
                )
                .collect())
        })
    }
}
