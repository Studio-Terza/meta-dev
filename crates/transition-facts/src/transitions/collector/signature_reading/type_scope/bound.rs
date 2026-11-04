use syn::{PathSegment, TraitBoundModifier, TypeParamBound};

use lint_layout::{Name, Whole};

use super::TypeScope;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::resolved::Resolved;
use crate::wrapping::Wrapping;

#[derive(Debug)]
pub(super) struct Bound<'bound>(&'bound TypeParamBound);

impl<'bound> Bound<'bound> {
    pub(super) fn new(bound: &'bound TypeParamBound) -> Bound<'bound> {
        Bound(bound)
    }

    pub(super) fn endpoint(&self, scope: &TypeScope<'_>) -> Option<Resolved> {
        let traited = self.traited();
        let iterated = traited.filter(|last| Name::from(&last.ident).matches::<Whole>(&Wrapping::ITERATORS).is_some());
        iterated
            .map(|last| {
                scope.argument(&last.arguments).unwrap_or_else(|| Resolved::Found(Endpoint::from(Unnamed::Opaque)))
            })
            .or_else(|| traited.map(|last| Resolved::Found(Endpoint::new(Nature::Interface, Name::from(&last.ident)))))
    }

    fn traited(&self) -> Option<&'bound PathSegment> {
        match self.0 {
            TypeParamBound::Trait(found) => Some(found)
                .filter(|found| matches!(found.modifier, TraitBoundModifier::None))
                .and_then(|found| found.path.segments.last()),
            _ => None,
        }
    }
}
