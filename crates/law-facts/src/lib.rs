mod kind;
mod law;
mod law_path;
mod operators;
mod package;
mod reader;
mod traits;

pub use kind::{Kind, Property};
pub use law::Law;
pub use law_path::{LawPath, Side};
pub use operators::Operators;
pub use package::Package;

use lint_core::SourceFile;
use lint_layout::Name;
use reader::Reader;
use traits::Traits;

#[derive(Debug, Default)]
pub struct Laws<'file> {
    laws: Vec<Law<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Laws<'file> {
    fn from(files: &'file [SourceFile]) -> Laws<'file> {
        let traits = Traits::declared(files);
        files.iter().fold(Laws::default(), |laws, file| laws.read(file, &traits))
    }
}

impl<'file> Laws<'file> {
    #[must_use]
    pub fn naming(&self, package: Package<'_>, names: &[&Name]) -> Vec<&Law<'file>> {
        self.laws.iter().filter_map(|law| law.naming(package, names)).collect()
    }

    #[must_use]
    pub fn routing(&self, package: Package<'_>, owner: &Name, member: &Name) -> Vec<&Law<'file>> {
        self.laws
            .iter()
            .filter_map(|law| law.of_package(package))
            .filter(|law| law.routed(owner, member).is_some())
            .collect()
    }

    #[must_use]
    pub fn of_package(&self, package: Package<'_>) -> Vec<&Law<'file>> {
        self.laws.iter().filter_map(|law| law.of_package(package)).collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Law<'file>> {
        self.laws.iter()
    }

    fn read(mut self, file: &'file SourceFile, traits: &Traits<'file>) -> Laws<'file> {
        self.laws.extend(Reader::read(file, traits));
        file.test_modules().iter().fold(self, |laws, module| laws.read(module, traits))
    }
}
