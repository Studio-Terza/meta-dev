mod admission;
mod directory;
mod packages;
mod regrouped;
mod target;

use admission::Admission;
use directory::Directory;
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
    pub fn files(files: &impl Files, trees: &[&Tree<'_>]) -> Vec<Result<Vec<PathBuf>, Failure>> {
        Directory::files(files, &Self::canonical(trees))
    }

    pub fn under(files: &impl Files, trees: &[&Tree<'_>]) -> Vec<Result<Vec<Target>, Failure>> {
        let below = Directory::trees(files, &Self::canonical(trees));
        let directories: Vec<Vec<PathBuf>> =
            trees.iter().zip(&below).map(|(tree, below)| Self::searched(tree, below)).collect();
        let read = Regrouped::like(&directories, Manifest::read_many(files, &directories.concat()));
        let manifests = below
            .into_iter()
            .zip(read)
            .map(|(below, read)| {
                below.and_then(|_| read.into_iter().collect::<Result<Vec<Option<Manifest>>, Failure>>())
            })
            .map(|found| found.map(|found| found.into_iter().flatten().collect()))
            .collect();
        Packages::targets_of(files, trees, Admission::members_among(files, manifests))
    }

    fn searched(tree: &Tree<'_>, below: &Result<Vec<Directory>, Failure>) -> Vec<PathBuf> {
        below
            .iter()
            .flat_map(|below| iter::once(tree.canonical()).chain(below.iter().map(AsRef::as_ref)))
            .map(Path::to_path_buf)
            .chain(below.iter().flat_map(|_| tree.canonical().ancestors().skip(1).map(Path::to_path_buf)))
            .collect()
    }

    fn canonical(trees: &[&Tree<'_>]) -> Vec<PathBuf> {
        trees.iter().map(|tree| tree.canonical().to_path_buf()).collect()
    }
}
