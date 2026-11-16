mod admission;
mod directory;
mod packages;
mod regrouped;
mod target;

use admission::Admission;
use directory::Directory;
pub use directory::{Below, Listing, Surveyed};
use packages::Packages;
pub use regrouped::Regrouped;

pub use target::Target;

use std::iter;
use std::path::{Path, PathBuf};

use crate::tree::Tree;
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    pub fn surveyed(files: &impl Files, trees: &[&Tree<'_>]) -> Surveyed {
        Directory::shared(files, &Self::canonical(trees))
    }

    pub fn under(files: &impl Files, trees: &[&Tree<'_>], below: Below) -> Vec<Result<Vec<Target>, Failure>> {
        let walked = below.trees();
        let directories: Vec<Vec<PathBuf>> =
            trees.iter().zip(&walked).map(|(tree, found)| Self::searched(tree, found)).collect();
        let read = Regrouped::like(&directories, Manifest::read_many(files, &directories.concat()));
        let manifests = walked
            .into_iter()
            .zip(read)
            .map(|(searched, outcomes)| {
                searched.and_then(|_| outcomes.into_iter().collect::<Result<Vec<Option<Manifest>>, Failure>>())
            })
            .map(|found| found.map(|listed| listed.into_iter().flatten().collect()))
            .collect();
        Packages::targets_of(files, trees, Admission::members_among(files, manifests))
    }

    fn searched(tree: &Tree<'_>, below: &Result<Vec<Directory>, Failure>) -> Vec<PathBuf> {
        below
            .iter()
            .flat_map(|directories| iter::once(tree.canonical()).chain(directories.iter().map(AsRef::as_ref)))
            .map(Path::to_path_buf)
            .chain(below.iter().flat_map(|_| tree.canonical().ancestors().skip(1).map(Path::to_path_buf)))
            .collect()
    }

    fn canonical(trees: &[&Tree<'_>]) -> Vec<PathBuf> {
        trees.iter().map(|tree| tree.canonical().to_path_buf()).collect()
    }
}
