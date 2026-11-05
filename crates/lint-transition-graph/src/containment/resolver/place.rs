use std::collections::BTreeMap;

use lint_item_graph::Scope;
use lint_layout::Name;
use lint_placement::CrateName;

use super::super::member::Member;
use super::target::Target;

#[derive(Debug)]
pub(crate) struct Place<'found> {
    krate: &'found CrateName,
    module: &'found [Name],
    scope: Scope<'found>,
    functions: BTreeMap<&'found Name, Member<'found>>,
}

impl<'found> Place<'found> {
    pub(crate) fn new(
        krate: &'found CrateName,
        module: &'found [Name],
        scope: Scope<'found>,
        functions: BTreeMap<&'found Name, Member<'found>>,
    ) -> Place<'found> {
        Place { krate, module, scope, functions }
    }

    pub(crate) fn function(&self, name: &Name) -> Option<Member<'found>> {
        self.functions.get(name).copied()
    }

    pub(crate) fn target(&self, module: Option<Vec<Name>>) -> Target<'found> {
        Target::new(self.krate, module, Some(self.module))
    }

    pub(crate) fn within(&self, base: &[Name], tail: &[Name]) -> Target<'found> {
        self.target(Some(base.iter().chain(tail).cloned().collect()))
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
