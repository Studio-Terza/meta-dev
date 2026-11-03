use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::dependency::Dependency;
use crate::shape::Shape;

#[derive(Clone, Debug)]
pub struct TypeFact<'file> {
    krate: &'file CrateName,
    name: Name,
    site: Site<'file>,
    derives: Vec<Name>,
    shape: Shape,
    dependencies: Vec<Dependency<'file>>,
}

impl<'file> TypeFact<'file> {
    pub(crate) fn declared(
        krate: &'file CrateName,
        name: Name,
        site: Site<'file>,
        derives: Vec<Name>,
    ) -> TypeFact<'file> {
        TypeFact { krate, name, site, derives, shape: Shape::Other, dependencies: Vec::new() }
    }

    pub(crate) fn depending_on(mut self, dependencies: Vec<Dependency<'file>>) -> TypeFact<'file> {
        self.dependencies = dependencies;
        self
    }

    pub(crate) fn shaped(mut self, shape: Shape) -> TypeFact<'file> {
        self.shape = shape;
        self
    }

    pub(crate) fn named_like(&self, other: &TypeFact<'_>) -> Option<&TypeFact<'file>> {
        (self.krate == other.krate).then_some(self).filter(|_| self.name == other.name)
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
    pub fn dependencies(&self) -> &[Dependency<'file>] {
        &self.dependencies
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
