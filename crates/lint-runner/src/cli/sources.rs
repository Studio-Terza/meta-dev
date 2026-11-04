use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Sources<'run, F> {
    files: &'run F,
    roots: &'run [PathBuf],
}

impl<'run, F> Sources<'run, F> {
    pub(crate) fn new(files: &'run F, roots: &'run [PathBuf]) -> Sources<'run, F> {
        Sources { files, roots }
    }

    pub(crate) fn files(&self) -> &'run F {
        self.files
    }

    pub(crate) fn roots(&self) -> &'run [PathBuf] {
        self.roots
    }
}
