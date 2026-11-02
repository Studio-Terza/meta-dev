use lint_layout::Name;

use crate::arity::Arity;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Method {
    name: Name,
    arity: Arity,
    returns: Option<Name>,
}

impl Method {
    pub(crate) fn new(name: Name, arity: Arity, returns: Option<Name>) -> Method {
        Method { name, arity, returns }
    }

    #[must_use]
    pub fn arity(&self) -> Arity {
        self.arity
    }

    #[must_use]
    pub fn returns(&self) -> Option<&Name> {
        self.returns.as_ref()
    }
}

impl AsRef<Name> for Method {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
