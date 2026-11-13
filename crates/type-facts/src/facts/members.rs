use std::collections::BTreeMap;

use lint_layout::Name;
use lint_placement::CrateName;

use crate::implementation::Implementation;
use crate::method::Method;

#[derive(Clone, Debug, Default)]
pub(crate) struct Members<'file> {
    methods: Vec<Method>,
    implementations: Vec<Implementation<'file>>,
}

impl<'file> Members<'file> {
    pub(crate) fn extend(&mut self, implementation: Option<Implementation<'file>>, methods: Vec<Method>) {
        self.methods.extend(methods);
        self.implementations.extend(implementation);
    }

    pub(crate) fn absorb(&mut self, later: Members<'file>) {
        self.methods.extend(later.methods);
        self.implementations.extend(later.implementations);
    }

    pub(crate) fn methods(&self) -> &[Method] {
        &self.methods
    }

    pub(crate) fn implementations(&self) -> &[Implementation<'file>] {
        &self.implementations
    }
}

type Crated<'file> = BTreeMap<&'file CrateName, BTreeMap<Name, Members<'file>>>;

impl<'file> Members<'file> {
    pub(crate) fn absorbed_into(
        mut crates: Crated<'file>,
        krate: &'file CrateName,
        named: BTreeMap<Name, Members<'file>>,
    ) -> Crated<'file> {
        let held = crates.entry(krate).or_default();
        named.into_iter().fold(held, |types, (name, found)| {
            types.entry(name).or_default().absorb(found);
            types
        });
        crates
    }
}
