use std::path::Path;

use lint_core::SourceFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Package<'file>(&'file Path);

impl<'file> Package<'file> {
    pub(crate) fn containing(file: &'file SourceFile) -> Package<'file> {
        Package(file.placement().package().root().directory())
    }
}
