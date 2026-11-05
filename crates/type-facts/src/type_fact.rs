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
    variants: Option<Vec<Name>>,
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
        TypeFact { krate, name, site, derives, variants: None, shape: Shape::Other, dependencies: Vec::new() }
    }

    pub(crate) fn depending_on(mut self, dependencies: Vec<Dependency<'file>>) -> TypeFact<'file> {
        self.dependencies = dependencies;
        self
    }

    pub(crate) fn with_variants(mut self, variants: Vec<Name>) -> TypeFact<'file> {
        self.variants = Some(variants);
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

    pub(crate) fn as_enum(&self) -> Option<&TypeFact<'file>> {
        self.variants.as_ref().map(|_| self)
    }

    #[must_use]
    pub fn variant(&self, wanted: &Name) -> Option<&Name> {
        self.variants.iter().flatten().find(|variant| *variant == wanted)
    }

    #[must_use]
    pub fn derives(&self) -> &[Name] {
        &self.derives
    }
}

impl<'file> AsRef<[Dependency<'file>]> for TypeFact<'file> {
    fn as_ref(&self) -> &[Dependency<'file>] {
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
