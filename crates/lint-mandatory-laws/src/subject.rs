use law_facts::Package;
use lint_core::FileHead;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Subject<'file> {
    package: Package<'file>,
    name: Name,
}

impl<'file> Subject<'file> {
    pub(crate) fn new(head: &'file FileHead, name: Name) -> Subject<'file> {
        Subject { package: Package::from(head), name }
    }

    pub(crate) fn package(&self) -> Package<'file> {
        self.package
    }

    pub(crate) fn owner(&self) -> &Name {
        &self.name
    }
}
