mod repo_files;

use std::path::Path;

use file_ports::Files;
use lint_core::RepoFile;
use lint_layout::Failure;
use lint_roots::{Roots, Target, Tree};

use super::located::{Located, Node};
use repo_files::RepoFiles;

#[derive(Debug)]
pub struct Staged {
    targets: Vec<Target>,
    repo_files: Vec<RepoFile>,
}

impl Staged {
    pub fn under(files: &impl Files, trees: &[&Tree]) -> Vec<Result<Staged, Failure>> {
        let targets = Roots::under(files, trees);
        let repo_files = RepoFiles::under(files, trees);
        trees
            .iter()
            .zip(targets)
            .zip(repo_files)
            .map(|((tree, targets), repo_files)| {
                let targets = targets?;
                if targets.is_empty() {
                    Err(tree.without_sources())
                } else {
                    Ok(Staged { targets, repo_files: repo_files? })
                }
            })
            .collect()
    }

    #[must_use]
    pub fn starts<'tree>(&self, tree: &'tree Tree) -> Vec<Node<'tree>> {
        self.targets
            .iter()
            .map(|root| {
                let file = root.clone().into_file();
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                Located::new(tree, root.package(), &file).node(&children)
            })
            .collect()
    }

    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    #[must_use]
    pub fn into_repo_files(self) -> Vec<RepoFile> {
        self.repo_files
    }
}
