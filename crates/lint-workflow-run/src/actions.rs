use std::path::Path;

use lint_core::RepoFile;

use crate::judged::Judged;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Actions<'files>(&'files [RepoFile]);

impl<'files> Actions<'files> {
    pub(crate) fn among(files: &'files [RepoFile]) -> Actions<'files> {
        Actions(files)
    }

    pub(crate) fn resolving(self, target: &Path) -> Option<&'files RepoFile> {
        self.0.iter().find_map(|file| Self::provided(file, target))
    }

    fn provided<'file>(file: &'file RepoFile, target: &Path) -> Option<&'file RepoFile> {
        let rest = file.below(target)?;
        Judged::MANIFESTS.iter().find(|manifest| rest == Path::new(manifest)).map(|_| file).or_else(|| {
            (Judged::judging(file) == Some(Judged::Workflow) && rest.as_os_str().is_empty()).then_some(file)
        })
    }
}
