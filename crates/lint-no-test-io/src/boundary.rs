use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct TestTarget;

impl TestTarget {
    const TESTS: &'static str = "tests";
    pub(crate) fn rooted(root: &Path) -> Option<&Path> {
        root.ancestors().skip(1).find(|directory| directory.file_name() == Some(OsStr::new(Self::TESTS))).map(|_| root)
    }
}
