use syntax::{AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, Type};

use graph::TypeRef;

use super::signature::{InScope, Written};

#[derive(Clone, Copy, Debug)]
pub(super) struct TraitPath<'syn>(&'syn Path);

impl<'syn> From<&'syn Path> for TraitPath<'syn> {
    fn from(path: &'syn Path) -> TraitPath<'syn> {
        TraitPath(path)
    }
}

impl TraitPath<'_> {
    pub(super) fn arguments(self, scope: &InScope) -> Vec<TypeRef<Written>> {
        let last = self.0.segments.last().map(|segment| &segment.arguments);
        last.and_then(TraitPath::angled)
            .into_iter()
            .flat_map(|bracketed| &bracketed.args)
            .filter_map(TraitPath::typed)
            .map(|ty| scope.lowered(ty))
            .collect()
    }

    fn angled(arguments: &PathArguments) -> Option<&AngleBracketedGenericArguments> {
        match arguments {
            PathArguments::AngleBracketed(angled) => Some(angled),
            PathArguments::None | PathArguments::Parenthesized(_) => None,
        }
    }

    fn typed(argument: &GenericArgument) -> Option<&Type> {
        match argument {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }
    }
}
