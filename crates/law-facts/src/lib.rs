mod kind;
mod law;
mod package;
mod reader;

use std::collections::BTreeSet;

pub use kind::{Kind, Property};
pub use law::Law;
pub use package::Package;

use lint_core::SourceFile;
use lint_layout::Name;
use reader::Reader;

#[derive(Debug, Default)]
pub struct Laws<'file> {
    laws: Vec<Law<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Laws<'file> {
    fn from(files: &'file [SourceFile]) -> Laws<'file> {
        files.iter().fold(Laws::default(), Laws::read)
    }
}

impl<'file> Laws<'file> {
    #[must_use]
    pub fn naming(&self, package: Package<'_>, names: &[&Name]) -> Vec<&Law<'file>> {
        self.laws.iter().filter_map(|law| law.naming(package, names)).collect()
    }

    #[must_use]
    pub fn properties_naming(&self, package: Package<'_>, names: &[&Name]) -> BTreeSet<Property> {
        self.naming(package, names).into_iter().filter_map(|law| law.kind().property()).collect()
    }

    fn read(mut self, file: &'file SourceFile) -> Laws<'file> {
        self.laws.extend(Reader::read(file));
        file.test_modules().iter().fold(self, Laws::read)
    }
}
