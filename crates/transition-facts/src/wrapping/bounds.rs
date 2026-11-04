use syn::punctuated::Punctuated;
use syn::token::Plus;
use syn::{PathSegment, TraitBoundModifier, TypeParamBound};

use lint_layout::{Name, Whole};

use super::Wrapping;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds<'bound>(&'bound Punctuated<TypeParamBound, Plus>);

impl<'bound> Bounds<'bound> {
    pub(super) fn new(bounds: &'bound Punctuated<TypeParamBound, Plus>) -> Bounds<'bound> {
        Bounds(bounds)
    }

    pub(super) fn wrapping(self) -> Wrapping {
        Self::wrapping_of(self.0.iter())
    }

    pub(crate) fn wrapping_of<'found>(mut bounds: impl Iterator<Item = &'found TypeParamBound>) -> Wrapping {
        bounds.find_map(Self::iterating).map_or(Wrapping::Plain, |_| Wrapping::Many)
    }

    pub(crate) fn iterating(bound: &TypeParamBound) -> Option<&PathSegment> {
        match bound {
            TypeParamBound::Trait(found) => Some(found)
                .filter(|found| matches!(found.modifier, TraitBoundModifier::None))
                .and_then(|found| found.path.segments.last())
                .filter(|last| Name::from(&last.ident).matches::<Whole>(&Wrapping::ITERATORS).is_some()),
            _ => None,
        }
    }
}
