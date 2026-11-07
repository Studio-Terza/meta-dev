use syn::{PathSegment, TraitBoundModifier, TypeParamBound};

use lint_layout::Name;

use super::TypeScope;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::resolved::Resolved;
use crate::wrapping::Bounds;

#[derive(Debug)]
pub(super) struct Bound<'bound>(&'bound TypeParamBound);

impl<'bound> Bound<'bound> {
    pub(super) const fn new(bound: &'bound TypeParamBound) -> Self {
        Bound(bound)
    }

    pub(super) fn endpoint(&self, scope: &TypeScope<'_>) -> Option<Resolved> {
        let traited = self.traited();
        Bounds::iterating(self.0)
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
