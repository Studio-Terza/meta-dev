use std::ffi::OsStr;
use std::path::Path;

use lint_core::RepoFile;

use crate::WorkflowRun;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Actions<'files>(&'files [RepoFile]);

impl<'files> Actions<'files> {
    const MANIFESTS: [&'static str; 2] = ["action.yml", "action.yaml"];

    pub(crate) fn among(files: &'files [RepoFile]) -> Actions<'files> {
        Actions(files)
    }

    pub(crate) fn resolving(self, target: &Path) -> Option<&'files RepoFile> {
        self.0.iter().find_map(|file| Self::provided(file, target))
    }

    pub(crate) fn manifest(file: &RepoFile) -> Option<&RepoFile> {
        file.shown()
            .file_name()
            .filter(|name| Self::MANIFESTS.iter().any(|manifest| *name == OsStr::new(manifest)))
            .map(|_| file)
    }

    fn provided<'file>(file: &'file RepoFile, target: &Path) -> Option<&'file RepoFile> {
        let rest = file.below(target)?;
        Self::MANIFESTS
            .iter()
            .find(|manifest| rest == Path::new(manifest))
            .map(|_| file)
            .or_else(|| WorkflowRun::workflow(file).filter(|_| rest.as_os_str().is_empty()))
    }
}
