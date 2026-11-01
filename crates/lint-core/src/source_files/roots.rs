mod admission;
mod directory;
mod package;
mod target;

use admission::Admission;
use directory::Directory;
use package::Package;

pub use target::Target;

use super::tree::Tree;
use crate::failure::Failure;
use crate::manifest::Manifest;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    pub fn under(tree: &Tree) -> Result<Vec<Target>, Failure> {
        let below = Directory::new(tree.canonical().to_path_buf()).tree()?;
        let above = tree.canonical().ancestors().skip(1).map(|ancestor| Directory::new(ancestor.to_path_buf()));
        let manifests = below.into_iter().chain(above).try_fold(Vec::new(), |found, directory| {
            Ok(found.into_iter().chain(Manifest::read(directory.as_ref())?.filter(Manifest::is_package)).collect())
        })?;
        Admission::members_among(manifests)?.iter().try_fold(Vec::new(), |roots, manifest| {
            Ok(roots
                .into_iter()
                .chain(
                    Package::in_directory(manifest.directory().to_path_buf(), manifest.declared_roots())
                        .targets_under(tree)?,
                )
                .collect())
        })
    }
}
