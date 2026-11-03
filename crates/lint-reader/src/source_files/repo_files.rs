use file_ports::Files;
use lint_core::{Contents, RepoFile};
use lint_layout::Failure;
use lint_roots::Roots;
use lint_roots::Tree;

#[derive(Debug)]
pub struct RepoFiles;

impl RepoFiles {
    pub fn under(files: &impl Files, tree: &Tree) -> Result<Vec<RepoFile>, Failure> {
        Ok(Roots::files(files, tree)?
            .iter()
            .map(|file| {
                let inside = file.strip_prefix(tree.canonical()).unwrap_or(file);
                RepoFile::new(tree.display(file), inside, Contents::from(files.read(file)))
            })
            .collect())
    }
}
