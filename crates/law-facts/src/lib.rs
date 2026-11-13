mod kind;
mod law;
pub mod law_path;
mod operators;
mod package;
mod reader;
mod traits;
mod uses;

pub use kind::{Kind, Property};
pub use law::Law;
pub use operators::Operators;
pub use package::Package;

use lint_core::{Parsed, SourceFile};
use lint_layout::Name;
use reader::Reader;
pub use traits::Traits;

#[derive(Debug, Default)]
pub struct Laws<'file> {
    laws: Vec<Law<'file>>,
}

impl<'file> From<&'file [SourceFile]> for Laws<'file> {
    fn from(files: &'file [SourceFile]) -> Laws<'file> {
        let traits: Traits<'file> = files.iter().map(Parsed::from).map(Traits::from).collect();
        files.iter().map(|file| Laws::read(Parsed::from(file), &traits)).collect()
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

    #[must_use]
    pub fn read(parsed: Parsed<'_, 'file>, traits: &Traits<'file>) -> Laws<'file> {
        let tested = parsed.test_modules().into_iter().map(|module| Laws::read(module, traits));
        Laws { laws: Reader::read(parsed, traits).into_iter().chain(tested.flat_map(|laws| laws.laws)).collect() }
    }
}

impl<'file> FromIterator<Laws<'file>> for Laws<'file> {
    fn from_iter<Parts: IntoIterator<Item = Laws<'file>>>(parts: Parts) -> Laws<'file> {
        Laws { laws: parts.into_iter().flat_map(|part| part.laws).collect() }
    }
}
