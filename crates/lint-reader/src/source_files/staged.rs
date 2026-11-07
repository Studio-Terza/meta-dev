mod repo_files;

use std::io;
use std::panic;
use std::path::Path;
use std::thread::{self, ScopedJoinHandle};

use file_ports::Files;
use lint_core::RepoFile;
use lint_layout::Failure;
use lint_roots::{Roots, Target, Tree};

use super::located::{Located, Node};
use repo_files::RepoFiles;

type Listed = Vec<Result<Vec<RepoFile>, Failure>>;

#[derive(Debug)]
pub struct Staged {
    targets: Vec<Target>,
    repo_files: Vec<RepoFile>,
}

impl Staged {
    pub fn under(files: &(impl Files + Sync), trees: &[&Tree<'_>]) -> Vec<Result<Self, Failure>> {
        let (targets, repo_files) = thread::scope(|scope| {
            let listing = thread::Builder::new().spawn_scoped(scope, || RepoFiles::under(files, trees));
            (Roots::under(files, trees), Self::listed(listing, files, trees))
        });
        trees
            .iter()
            .zip(targets)
            .zip(repo_files)
            .map(|((tree, targets), repo_files)| {
                let targets = targets?;
                targets.first().ok_or_else(|| tree.without_sources())?;
                Ok(Self { targets, repo_files: repo_files? })
            })
            .collect()
    }

    fn listed(listing: io::Result<ScopedJoinHandle<'_, Listed>>, files: &impl Files, trees: &[&Tree<'_>]) -> Listed {
        match listing {
            Ok(listing) => listing.join().unwrap_or_else(|payload| panic::resume_unwind(payload)),
            Err(_) => RepoFiles::under(files, trees),
        }
    }

    #[must_use]
    pub fn starts<'tree>(&self, tree: &'tree Tree<'tree>) -> Vec<Node<'tree>> {
        self.targets
            .iter()
            .flat_map(|target| {
                target.files().iter().map(move |file| {
                    let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                    Located::new(tree, target.package(), file).node(&children)
                })
            })
            .collect()
    }

    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    #[must_use]
    pub fn repo_files(&self) -> &[RepoFile] {
        &self.repo_files
    }
}
