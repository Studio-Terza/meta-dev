use std::path::PathBuf;

use lint_core::Readout;
use workspace_metadata::Metadata;

#[derive(Debug)]
pub(crate) struct Sources<'run, F> {
    files: &'run F,
    roots: &'run [PathBuf],
    workspaces: &'run [Metadata],
}

impl<'run, F> Sources<'run, F> {
    pub(crate) fn new(files: &'run F, roots: &'run [PathBuf], workspaces: &'run [Metadata]) -> Sources<'run, F> {
        Sources { files, roots, workspaces }
    }

    pub(crate) fn files(&self) -> &'run F {
        self.files
    }

    pub(crate) fn roots(&self) -> &'run [PathBuf] {
        self.roots
    }

    pub(crate) fn described(&self, readout: Readout) -> Readout {
        readout.with_workspaces(self.workspaces.to_vec())
    }
}
