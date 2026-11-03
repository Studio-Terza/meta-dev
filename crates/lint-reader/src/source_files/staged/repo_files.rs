use std::path::PathBuf;

use file_ports::{Bytes, FileFailure, Files};
use lint_core::{Contents, RepoFile};
use lint_layout::Failure;
use lint_roots::Regrouped;
use lint_roots::Roots;
use lint_roots::Tree;

#[derive(Debug)]
pub struct RepoFiles;

impl RepoFiles {
    pub fn under(files: &impl Files, trees: &[&Tree<'_>]) -> Vec<Result<Vec<RepoFile>, Failure>> {
        let listed = Roots::files(files, trees);
        let paths: Vec<Vec<PathBuf>> =
            listed.iter().map(|found| found.as_ref().map_or_else(|_| Vec::new(), Clone::clone)).collect();
        let contents = Regrouped::like(&paths, files.read_many(&paths.concat()));
        trees
            .iter()
            .zip(listed)
            .zip(contents)
            .map(|((tree, listed), contents)| listed.map(|listed| Self::found(tree, &listed, contents)))
            .collect()
    }

    fn found(tree: &Tree<'_>, listed: &[PathBuf], contents: Vec<Result<Bytes, FileFailure>>) -> Vec<RepoFile> {
        listed
            .iter()
            .zip(contents)
            .map(|(file, read)| {
                let inside = file.strip_prefix(tree.canonical()).unwrap_or(file);
                RepoFile::new(tree.display(file), inside, Contents::from(read))
            })
            .collect()
    }
}
