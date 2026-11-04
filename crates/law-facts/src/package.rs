use std::path::Path;

use lint_core::SourceFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Package<'file>(&'file Path);

impl<'file> Package<'file> {
    #[must_use]
    pub fn containing(file: &'file SourceFile) -> Package<'file> {
        Package(file.placement().package().root().directory())
    }
}
