use std::ffi::OsStr;
use std::path::Path;

use crate::source_file::SourceFile;
use lint_placement::Role;

#[derive(Clone, Copy, Debug)]
pub struct TestFile<'file>(&'file SourceFile);

impl<'file> TestFile<'file> {
    const TESTS: &'static str = "tests";

    #[must_use]
    pub fn tested(file: &'file SourceFile) -> Option<TestFile<'file>> {
        let placement = file.placement();
        (placement.role() == Role::Target
            && placement.crate_root().directory().parent().and_then(Path::file_name) == Some(OsStr::new(Self::TESTS)))
        .then_some(TestFile(file))
    }

    #[must_use]
    pub fn file(self) -> &'file SourceFile {
        self.0
    }
}
