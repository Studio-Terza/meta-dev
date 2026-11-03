use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::shape::Shape;

#[derive(Clone, Debug)]
pub struct TypeFact<'file> {
    krate: &'file CrateName,
    name: Name,
    site: Site<'file>,
    derives: Vec<Name>,
    shape: Shape,
}

impl<'file> TypeFact<'file> {
    pub(crate) fn declared(
        krate: &'file CrateName,
        name: Name,
        site: Site<'file>,
        derives: Vec<Name>,
    ) -> TypeFact<'file> {
        TypeFact { krate, name, site, derives, shape: Shape::Other }
    }

    pub(crate) fn shaped(mut self, shape: Shape) -> TypeFact<'file> {
        self.shape = shape;
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
}

impl AsRef<CrateName> for TypeFact<'_> {
    fn as_ref(&self) -> &CrateName {
        self.krate
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
