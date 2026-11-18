use std::path::PathBuf;

use crate::source_files::ignored::Ignored;
use file_ports::{Bytes, FileFailure, Files};
use invariants::Ignoring;
use lint_core::{Contents, RepoFile};
use lint_layout::Failure;
use lint_roots::Regrouped;
use lint_roots::{Listing, Tree};

#[derive(Debug)]
pub struct RepoFiles;

impl RepoFiles {
    pub fn under(files: &impl Files, trees: &[&Tree<'_>], listing: Listing) -> Vec<Result<Vec<RepoFile>, Failure>> {
        let listed = listing.files();
        let paths: Vec<Vec<PathBuf>> = listed
            .iter()
            .map(|found| found.as_ref().ignoring(Ignored::Unlisted).map_or_default(Clone::clone))
            .collect();
        let contents = Regrouped::like(&paths, files.read_many(&paths.concat()));
        trees
            .iter()
            .zip(listed)
            .zip(contents)
            .map(|((tree, outcome), read)| outcome.map(|entries| Self::found(tree, &entries, read)))
            .collect()
    }

    fn found(tree: &Tree<'_>, listed: &[PathBuf], contents: Vec<Result<Bytes, FileFailure>>) -> Vec<RepoFile> {
        listed
            .iter()
            .zip(contents)
            .map(|(file, read)| {
                let inside = file.strip_prefix(tree.canonical()).ignoring(Ignored::Outside).unwrap_or(file);
                RepoFile::new(tree.display(file), inside, Contents::from(read))
            })
            .collect()
    }
}
