use std::ffi::OsStr;
use std::path::Path;

use lint_core::RepoFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Judged {
    Workflow,
    Action,
}

impl Judged {
    pub(crate) const MANIFESTS: [&'static str; 2] = ["action.yml", "action.yaml"];
    const WORKFLOWS: &'static str = ".github/workflows";
    const EXTENSIONS: [&'static str; 2] = ["yml", "yaml"];

    pub(crate) fn judging(file: &RepoFile) -> Option<Self> {
        let workflow = file
            .below(Path::new(Self::WORKFLOWS))
            .and(file.shown().extension())
            .filter(|extension| Self::EXTENSIONS.iter().any(|known| *extension == *known))
            .map(|_| Self::Workflow);
        let action = file
            .shown()
            .file_name()
            .filter(|name| Self::MANIFESTS.iter().any(|manifest| *name == OsStr::new(manifest)))
            .map(|_| Self::Action);
        workflow.or(action)
    }
}
