use std::collections::BTreeMap;

use super::super::member::Member;
use super::super::unit::Unit;
use super::target::Target;
use lint_item_graph::Scope;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Place<'found> {
    unit: Unit<'found>,
    module: &'found [Name],
    scope: Scope<'found>,
    functions: BTreeMap<&'found Name, Member<'found>>,
}

impl<'found> Place<'found> {
    pub(crate) fn new(
        unit: Unit<'found>,
        module: &'found [Name],
        scope: Scope<'found>,
        functions: BTreeMap<&'found Name, Member<'found>>,
    ) -> Place<'found> {
        Place { unit, module, scope, functions }
    }

    pub(crate) fn function(&self, name: &Name) -> Option<Member<'found>> {
        self.functions.get(name).copied()
    }

    pub(crate) fn target(&self, module: Option<Vec<Name>>) -> Target<'found> {
        Target::new(self.unit, module, Some(self.module))
    }

    pub(crate) fn within(&self, base: &[Name], tail: &[Name]) -> Target<'found> {
        self.target(Some(base.iter().chain(tail).cloned().collect()))
    }

    pub(crate) fn unit(&self) -> Unit<'found> {
        self.unit
    }

    pub(crate) fn module(&self) -> &'found [Name] {
        self.module
    }

    pub(crate) fn scope(&self) -> &Scope<'found> {
        &self.scope
    }
}
