use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::implementation::Implementation;
use crate::method::Method;
use crate::shape::Shape;

#[derive(Clone, Debug)]
pub struct TypeFact {
    krate: CrateName,
    name: Name,
    site: Site,
    derives: Vec<Name>,
    shape: Shape,
    methods: Vec<Method>,
    implementations: Vec<Implementation>,
}

impl TypeFact {
    pub(crate) fn declared(krate: CrateName, name: Name, site: Site, derives: Vec<Name>) -> TypeFact {
        TypeFact { krate, name, site, derives, shape: Shape::Other, methods: Vec::new(), implementations: Vec::new() }
    }

    pub(crate) fn shaped(mut self, shape: Shape) -> TypeFact {
        self.shape = shape;
        self
    }

    pub(crate) fn with_methods(mut self, found: impl IntoIterator<Item = Method>) -> TypeFact {
        self.methods.extend(found);
        self
    }

    pub(crate) fn with_implementation(mut self, found: Option<Implementation>) -> TypeFact {
        self.implementations.extend(found);
        self
    }

    pub(crate) fn is_named_like(&self, other: &TypeFact) -> bool {
        self.krate == other.krate && self.name == other.name
    }

    #[must_use]
    pub fn shape(&self) -> Shape {
        self.shape
    }

    #[must_use]
    pub fn derives(&self) -> &[Name] {
        &self.derives
    }

    #[must_use]
    pub fn methods(&self) -> &[Method] {
        &self.methods
    }

    #[must_use]
    pub fn implementations(&self) -> &[Implementation] {
        &self.implementations
    }
}

impl AsRef<CrateName> for TypeFact {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Name> for TypeFact {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<Site> for TypeFact {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
