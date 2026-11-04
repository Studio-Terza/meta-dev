use law_facts::Package;
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Subject<'file> {
    package: Package<'file>,
    name: Name,
}

impl<'file> Subject<'file> {
    pub(crate) fn new(file: &'file SourceFile, name: Name) -> Subject<'file> {
        Subject { package: Package::containing(file), name }
    }

    pub(crate) fn package(&self) -> Package<'file> {
        self.package
    }

    pub(crate) fn owner(&self) -> &Name {
        &self.name
    }
}
