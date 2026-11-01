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

    pub(crate) fn resolves(self, target: &Path) -> bool {
        self.0.iter().any(|file| Self::provides(file, target))
    }

    pub(crate) fn is_manifest(file: &RepoFile) -> bool {
        file.shown().file_name().is_some_and(|name| Self::MANIFESTS.iter().any(|manifest| name == OsStr::new(manifest)))
    }

    fn provides(file: &RepoFile, target: &Path) -> bool {
        file.below(target).is_some_and(|rest| {
            Self::MANIFESTS.iter().any(|manifest| rest == Path::new(manifest))
                || (rest.as_os_str().is_empty() && WorkflowRun::is_workflow(file))
        })
    }
}
