mod crate_sources;
mod declared;
mod described;
mod ignored;
mod located;
mod modules;
mod shared;
mod staged;
mod unlocated;

use std::iter;
use std::path::{Path, PathBuf};

pub use crate_sources::CrateSources;
pub use described::Described;
use file_ports::Files;
use lint_core::Readout;
use lint_layout::Failure;
use lint_placement::Packages;
use lint_roots::{Regrouped, Target, Tree};
pub use shared::{Crew, Partition, Reader, Reads, SharedRead};
use staged::Staged;
pub use unlocated::Unlocated;

#[derive(Debug)]
pub struct Stage<'given> {
    trees: Vec<Tree<'given>>,
    staged: Vec<Vec<Staged>>,
    packages: Packages,
}

impl Stage<'_> {
    #[must_use]
    pub fn package_directories(&self) -> Vec<&Path> {
        self.staged.iter().flatten().flat_map(Staged::targets).map(Target::package).collect()
    }
}

#[derive(Debug)]
pub struct SourceFiles;

impl SourceFiles {
    pub fn read(files: &(impl Files + Sync), root: &Path) -> Result<Readout, Failure> {
        SourceFiles::read_all(files, &[root.to_path_buf()])
    }

    pub fn read_all(files: &(impl Files + Sync), roots: &[PathBuf]) -> Result<Readout, Failure> {
        SourceFiles::staged(files, roots).map(|stage| SourceFiles::read_staged(files, &stage))
    }

    pub fn staged<'given>(files: &(impl Files + Sync), roots: &'given [PathBuf]) -> Result<Stage<'given>, Failure> {
        let opened = Tree::open_many(files, roots);
        let shape: Vec<Vec<&Tree<'_>>> = opened.iter().map(|group| group.iter().collect()).collect();
        let regrouped = Regrouped::like(&shape, Staged::under(files, &shape.concat()));
        let (trees, staged): (Vec<Tree<'_>>, Vec<Vec<Staged>>) = opened
            .into_iter()
            .zip(regrouped)
            .map(|(attempt, stages)| Ok((attempt?, stages.into_iter().collect::<Result<Vec<Staged>, Failure>>()?)))
            .collect::<Result<Vec<(Tree<'_>, Vec<Staged>)>, Failure>>()?
            .into_iter()
            .unzip();
        let packages: Vec<&Path> = staged.iter().flatten().flat_map(Staged::targets).map(Target::package).collect();
        Ok(Stage { packages: Packages::read(files, &packages), trees, staged })
    }

    #[must_use]
    pub fn read_staged<'tree>(files: &impl Files, stage: &'tree Stage<'tree>) -> Readout {
        SharedRead::new(stage).crew(iter::empty()).concluded(|coordinator, _| coordinator.read(files))
    }
}
