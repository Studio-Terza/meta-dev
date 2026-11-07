use lint_item_graph::Scope;
use lint_layout::Name;
use lint_placement::CrateName;

use super::target::Target;

#[derive(Debug)]
pub(crate) struct Place<'found> {
    krate: &'found CrateName,
    module: &'found [Name],
    scope: Scope<'found>,
}

impl<'found> Place<'found> {
    pub(crate) const fn new(krate: &'found CrateName, module: &'found [Name], scope: Scope<'found>) -> Self {
        Place { krate, module, scope }
    }

    pub(crate) const fn target(&self, module: Option<Vec<Name>>) -> Target<'found> {
        Target::new(self.krate, module, Some(self.module))
    }

    pub(crate) fn within(&self, base: &[Name], tail: &[Name]) -> Target<'found> {
        self.target(Some(base.iter().chain(tail).cloned().collect()))
    }

    pub(crate) const fn krate(&self) -> &'found CrateName {
        self.krate
    }

    pub(crate) const fn module(&self) -> &'found [Name] {
        self.module
    }

    pub(crate) const fn scope(&self) -> &Scope<'found> {
        &self.scope
    }
}
