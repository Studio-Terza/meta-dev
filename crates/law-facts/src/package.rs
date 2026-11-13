use std::path::Path;

use lint_core::{FileHead, SourceFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Package<'file>(&'file Path);

impl<'file> Package<'file> {
    #[must_use]
    pub fn containing(file: &'file SourceFile) -> Package<'file> {
        Package(file.head().placement().package().root().directory())
    }
}

impl<'head> From<&'head FileHead> for Package<'head> {
    fn from(head: &'head FileHead) -> Package<'head> {
        Package(head.placement().package().root().directory())
    }
}
