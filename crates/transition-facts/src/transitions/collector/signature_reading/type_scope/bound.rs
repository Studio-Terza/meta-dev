use syn::{TraitBoundModifier, TypeParamBound};

use lint_layout::{Name, Whole, Word, words};

use super::TypeScope;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::resolved::Resolved;

#[derive(Debug)]
pub(super) struct Bound<'bound>(&'bound TypeParamBound);

impl<'bound> Bound<'bound> {
    const ITERATORS: [Word<'static>; 4] =
        words!["Iterator", "IntoIterator", "DoubleEndedIterator", "ExactSizeIterator"];

    pub(super) fn new(bound: &'bound TypeParamBound) -> Bound<'bound> {
        Bound(bound)
    }

    pub(super) fn endpoint(&self, scope: &TypeScope<'_>) -> Option<Resolved> {
        let traited = match self.0 {
            TypeParamBound::Trait(found) if matches!(found.modifier, TraitBoundModifier::None) => {
                found.path.segments.last()
            }
            _ => None,
        };
        let iterated = traited.filter(|last| Name::from(&last.ident).matches::<Whole>(&Self::ITERATORS).is_some());
        match iterated {
            Some(last) => Some(
                scope.argument(&last.arguments).unwrap_or_else(|| Resolved::Found(Endpoint::from(Unnamed::Opaque))),
            ),
            None => traited.map(|last| Resolved::Found(Endpoint::new(Nature::Interface, Name::from(&last.ident)))),
        }
    }
}
