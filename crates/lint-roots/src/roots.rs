mod admission;
mod directory;
mod packages;
mod target;

use admission::Admission;
use directory::Directory;
use packages::Packages;

pub use target::Target;

use std::path::{Path, PathBuf};

use crate::regrouped::Regrouped;
use crate::tree::Tree;
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    pub fn files(files: &impl Files, trees: &[&Tree]) -> Vec<Result<Vec<PathBuf>, Failure>> {
        Directory::files(files, &Self::canonical(trees))
    }

    pub fn under(files: &impl Files, trees: &[&Tree]) -> Vec<Result<Vec<Target>, Failure>> {
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

    fn searched(tree: &Tree, below: &Result<Vec<Directory>, Failure>) -> Vec<PathBuf> {
        below
            .iter()
            .flatten()
            .map(|directory| directory.as_ref().to_path_buf())
            .chain(below.iter().flat_map(|_| tree.canonical().ancestors().skip(1).map(Path::to_path_buf)))
            .collect()
    }

    fn canonical(trees: &[&Tree]) -> Vec<PathBuf> {
        trees.iter().map(|tree| tree.canonical().to_path_buf()).collect()
    }
}
