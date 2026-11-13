use crate::sited::Sited;

use super::{Callable, Named, Projection, TypeRef};

#[derive(Clone, Copy, Debug)]
pub struct Grouped<'ty>(&'ty [TypeRef]);

#[derive(Clone, Copy, Debug)]
pub struct Bounds<'ty>(&'ty [TypeRef]);

#[derive(Clone, Copy, Debug)]
pub struct Element<'ty>(&'ty TypeRef);

pub trait Outline {
    type Kind;

    fn named(self, named: &Named<Sited>) -> Self::Kind;

    fn grouped(self, parts: Grouped<'_>) -> Self::Kind;

    fn sequence(self, element: Element<'_>) -> Self::Kind;

    fn bounded(self, bounds: Bounds<'_>) -> Self::Kind;

    fn projected(self, projection: &Projection<Sited>) -> Self::Kind;

    fn callable(self, callable: &Callable<Sited>) -> Self::Kind;

    fn other(self) -> Self::Kind;
}

impl<'ty> From<&'ty [TypeRef]> for Grouped<'ty> {
    fn from(parts: &'ty [TypeRef]) -> Grouped<'ty> {
        Grouped(parts)
    }
}

impl<'ty> From<&'ty [TypeRef]> for Bounds<'ty> {
    fn from(bounds: &'ty [TypeRef]) -> Bounds<'ty> {
        Bounds(bounds)
    }
}

impl<'ty> From<&'ty TypeRef> for Element<'ty> {
    fn from(element: &'ty TypeRef) -> Element<'ty> {
        Element(element)
    }
}

impl AsRef<[TypeRef]> for Grouped<'_> {
    fn as_ref(&self) -> &[TypeRef] {
        self.0
    }
}

impl AsRef<[TypeRef]> for Bounds<'_> {
    fn as_ref(&self) -> &[TypeRef] {
        self.0
    }
}

impl AsRef<TypeRef> for Element<'_> {
    fn as_ref(&self) -> &TypeRef {
        self.0
    }
}
