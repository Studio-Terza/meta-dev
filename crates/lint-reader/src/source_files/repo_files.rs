use lint_core::RepoFile;
use lint_layout::Failure;
use lint_roots::Roots;
use lint_roots::Tree;

#[derive(Debug)]
pub struct RepoFiles;

impl RepoFiles {
    pub fn under(tree: &Tree) -> Result<Vec<RepoFile>, Failure> {
        Ok(Roots::files(tree)?
            .iter()
            .map(|file| RepoFile::new(tree.display(file), file.strip_prefix(tree.canonical()).unwrap_or(file)))
            .collect())
    }
}
