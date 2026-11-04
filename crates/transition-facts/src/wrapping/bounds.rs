use syn::punctuated::Punctuated;
use syn::token::Plus;
use syn::{PathSegment, TypeParamBound};

use lint_layout::{Name, Whole};

use super::Wrapping;

#[derive(Clone, Copy, Debug)]
pub(super) struct Bounds<'bound>(&'bound Punctuated<TypeParamBound, Plus>);

impl<'bound> Bounds<'bound> {
    pub(super) fn new(bounds: &'bound Punctuated<TypeParamBound, Plus>) -> Bounds<'bound> {
        Bounds(bounds)
    }

    pub(super) fn wrapping(self) -> Wrapping {
        self.0
            .iter()
            .filter_map(Self::traited)
            .find(|last| Name::from(&last.ident).matches::<Whole>(&Wrapping::ITERATORS).is_some())
            .map_or(Wrapping::Plain, |_| Wrapping::Many)
    }

    fn traited(bound: &TypeParamBound) -> Option<&PathSegment> {
        match bound {
            TypeParamBound::Trait(found) => found.path.segments.last(),
            _ => None,
        }
    }
}
