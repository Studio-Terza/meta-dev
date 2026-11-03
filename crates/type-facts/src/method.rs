use lint_layout::Name;

use crate::method_shape::MethodShape;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Method {
    name: Name,
    returns: Option<Name>,
    shape: MethodShape,
}

impl Method {
    pub(crate) fn new(name: Name, returns: Option<Name>, shape: MethodShape) -> Method {
        Method { name, returns, shape }
    }

    #[must_use]
    pub fn returns(&self) -> Option<&Name> {
        self.returns.as_ref()
    }

    #[must_use]
    pub fn shape(&self) -> &MethodShape {
        &self.shape
    }
}

impl AsRef<Name> for Method {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}
