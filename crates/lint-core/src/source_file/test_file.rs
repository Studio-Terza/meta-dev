use std::ffi::OsStr;
use std::path::Path;

use lint_placement::Role;

use crate::source_file::SourceFile;

#[derive(Clone, Copy, Debug)]
pub struct TestFile<'file>(&'file SourceFile);

impl<'file> TestFile<'file> {
    #[must_use]
    pub fn file(self) -> &'file SourceFile {
        self.0
    }
}

impl SourceFile {
    const TESTS: &'static str = "tests";

    #[must_use]
    pub fn tested(&self) -> Option<TestFile<'_>> {
        (self.placement.role() == Role::Target
            && self.placement.crate_root().directory().parent().and_then(Path::file_name)
                == Some(OsStr::new(Self::TESTS)))
        .then_some(TestFile(self))
    }
}
