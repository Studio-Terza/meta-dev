mod boundary;
mod check;
mod reader;
mod test;

pub use check::Check;
pub use test::{Test, TestId};

use lint_core::SourceFile;
use reader::Reader;

#[derive(Debug, Default)]
pub struct Tests<'file> {
    tests: Vec<Test<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Tests<'file> {
    fn from(files: &'file [SourceFile]) -> Tests<'file> {
        files.iter().fold(Tests::default(), Tests::read)
    }
}

impl<'file> Tests<'file> {
    pub fn all(&self) -> impl Iterator<Item = &Test<'file>> {
        self.tests.iter()
    }

    fn read(mut self, file: &'file SourceFile) -> Tests<'file> {
        self.tests.extend(Reader::read(file));
        file.test_modules().iter().fold(self, Tests::read)
    }
}
