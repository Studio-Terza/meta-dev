use std::collections::BTreeMap;

use lint_layout::Name;
use lint_placement::CrateName;

use crate::facts::Members;
use crate::implementation::Implementation;
use crate::method::Method;

#[derive(Debug)]
pub(crate) struct ImplBlock<'file> {
    krate: &'file CrateName,
    target: Name,
    implementation: Option<Implementation>,
    methods: Vec<Method>,
}

impl<'file> ImplBlock<'file> {
    pub(crate) fn new(
        krate: &'file CrateName,
        target: Name,
        implementation: Option<Implementation>,
        methods: Vec<Method>,
    ) -> ImplBlock<'file> {
        ImplBlock { krate, target, implementation, methods }
    }

    pub(crate) fn added_to(
        self,
        mut members: BTreeMap<&'file CrateName, BTreeMap<Name, Members>>,
    ) -> BTreeMap<&'file CrateName, BTreeMap<Name, Members>> {
        members
            .entry(self.krate)
            .or_default()
            .entry(self.target)
            .or_default()
            .extend(self.implementation, self.methods);
        members
    }
}
