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
    pub(crate) fn new(krate: &'found CrateName, module: &'found [Name], scope: Scope<'found>) -> Place<'found> {
        Place { krate, module, scope }
    }

    pub(crate) fn target(&self, module: Option<Vec<Name>>) -> Target<'found> {
        Target::new(self.krate, module, Some(self.module))
    }

    pub(crate) fn krate(&self) -> &'found CrateName {
        self.krate
    }

    pub(crate) fn module(&self) -> &'found [Name] {
        self.module
    }

    pub(crate) fn scope(&self) -> &Scope<'found> {
        &self.scope
    }
}
