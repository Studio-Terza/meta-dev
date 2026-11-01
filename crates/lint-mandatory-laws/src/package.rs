use std::path::PathBuf;

use lint_core::SourceFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Package(PathBuf);

impl Package {
    pub(crate) fn containing(file: &SourceFile) -> Package {
        Package(file.placement().package().root().directory().to_path_buf())
    }
}
