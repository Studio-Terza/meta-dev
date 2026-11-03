use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::implementation::Implementation;
use crate::method::Method;
use crate::shape::Shape;

#[derive(Clone, Debug)]
pub struct TypeFact<'file> {
    krate: CrateName,
    name: Name,
    site: Site<'file>,
    derives: Vec<Name>,
    shape: Shape,
    methods: Vec<Method>,
    implementations: Vec<Implementation>,
}

impl<'file> TypeFact<'file> {
    pub(crate) fn declared(krate: CrateName, name: Name, site: Site<'file>, derives: Vec<Name>) -> TypeFact<'file> {
        TypeFact { krate, name, site, derives, shape: Shape::Other, methods: Vec::new(), implementations: Vec::new() }
    }

    pub(crate) fn shaped(mut self, shape: Shape) -> TypeFact<'file> {
        self.shape = shape;
        self
    }

    pub(crate) fn with_methods(mut self, found: impl IntoIterator<Item = Method>) -> TypeFact<'file> {
        self.methods.extend(found);
        self
    }

    pub(crate) fn with_implementation(mut self, found: Option<Implementation>) -> TypeFact<'file> {
        self.implementations.extend(found);
        self
    }

    pub(crate) fn named_like(&self, other: &TypeFact<'_>) -> Option<&TypeFact<'file>> {
        (self.krate == other.krate && self.name == other.name).then_some(self)
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

impl AsRef<CrateName> for TypeFact<'_> {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Name> for TypeFact<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for TypeFact<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
