use lint_layout::Name;
use lint_placement::CrateName;

use crate::implementation::Implementation;
use crate::method::Method;
use crate::type_fact::TypeFact;

#[derive(Clone, Debug)]
pub(crate) struct ImplBlock {
    krate: CrateName,
    target: Name,
    implementation: Option<Implementation>,
    methods: Vec<Method>,
}

impl ImplBlock {
    pub(crate) fn new(
        krate: CrateName,
        target: Name,
        implementation: Option<Implementation>,
        methods: Vec<Method>,
    ) -> ImplBlock {
        ImplBlock { krate, target, implementation, methods }
    }

    pub(crate) fn applied_to(&self, found: TypeFact) -> TypeFact {
        let krate: &CrateName = found.as_ref();
        let name: &Name = found.as_ref();
        if *krate == self.krate && *name == self.target {
            found.with_methods(self.methods.clone()).with_implementation(self.implementation.clone())
        } else {
            found
        }
    }
}
