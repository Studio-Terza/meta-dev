use lint_layout::Name;

use crate::method::Method;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Implementation {
    implemented: Name,
    argument: Option<Name>,
    methods: Vec<Method>,
}

impl Implementation {
    pub(crate) fn new(implemented: Name, argument: Option<Name>, methods: Vec<Method>) -> Implementation {
        Implementation { implemented, argument, methods }
    }

    #[must_use]
    pub fn argument(&self) -> Option<&Name> {
        self.argument.as_ref()
    }

    #[must_use]
    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

impl AsRef<Name> for Implementation {
    fn as_ref(&self) -> &Name {
        &self.implemented
    }
}
