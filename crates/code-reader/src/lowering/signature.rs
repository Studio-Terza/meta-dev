use syn::{GenericArgument, GenericParam, Generics, PathArguments, ReturnType, Type, TypePath};

use code_model::{Named, Resolved, TypeRef};
use lint_layout::{Name, Whole, Word, words};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

#[derive(Clone, Debug)]
pub(super) enum Written {
    Path(UsePath),
    Param(Name),
}

impl Written {
    pub(super) fn typed(ty: TypeRef<Written>, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> TypeRef {
        ty.mapped(&mut |written: Written| written.resolved(resolving, scope))
    }

    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Resolved {
        match self {
            Written::Path(path) => resolving.path(scope, &path, Domain::Types),
            Written::Param(name) => Resolved::Param(name),
        }
    }

    fn parameter(parameter: Name, path: &syn::Path) -> TypeRef<Written> {
        match path.segments.get(1) {
            Some(_) => TypeRef::Opaque,
            None => TypeRef::Named(Named::new(Written::Param(parameter), Vec::new())),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TypeParameter;

impl TypeParameter {
    fn named(parameter: &GenericParam) -> Option<Name> {
        match parameter {
            GenericParam::Type(declared) => Some(Name::from(&declared.ident)),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct InScope {
    parameters: Vec<Name>,
}

impl InScope {
    const SELF: [Word<'static>; 1] = words!["Self"];

    pub(super) fn extended(&self, generics: &Generics) -> InScope {
        let declared = generics.params.iter().filter_map(TypeParameter::named);
        InScope { parameters: self.parameters.iter().cloned().chain(declared).collect() }
    }

    pub(super) fn parameters(&self) -> Vec<Name> {
        self.parameters.clone()
    }

    pub(super) fn output(&self, output: &ReturnType) -> Option<TypeRef<Written>> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(self.lowered(ty)),
        }
    }

    pub(super) fn lowered(&self, ty: &Type) -> TypeRef<Written> {
        match ty {
            Type::Path(path) => self.type_path_lowered(path),
            Type::Tuple(tuple) => TypeRef::Tuple(tuple.elems.iter().map(|element| self.lowered(element)).collect()),
            Type::Paren(inner) => self.lowered(&inner.elem),
            _ => TypeRef::Opaque,
        }
    }

    fn type_path_lowered(&self, path: &TypePath) -> TypeRef<Written> {
        match path.qself {
            Some(_) => TypeRef::Opaque,
            None => self.path_lowered(&path.path),
        }
    }

    fn path_lowered(&self, path: &syn::Path) -> TypeRef<Written> {
        match path.segments.first().map(|first| Name::from(&first.ident)).and_then(|first| self.held(first)) {
            Some(parameter) => Written::parameter(parameter, path),
            None => TypeRef::Named(Named::new(Written::Path(UsePath::from(path)), self.arguments(path))),
        }
    }

    fn held(&self, first: Name) -> Option<Name> {
        let declared = self.parameters.iter().find(|parameter| **parameter == first).cloned();
        declared.or_else(|| first.matches::<Whole>(&Self::SELF).map(|_| first))
    }

    fn arguments(&self, path: &syn::Path) -> Vec<TypeRef<Written>> {
        let bracketed = Bracketed { scope: self };
        path.segments.last().map(|last| bracketed.listed(&last.arguments)).unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug)]
struct Bracketed<'scope> {
    scope: &'scope InScope,
}

impl Bracketed<'_> {
    fn listed(self, arguments: &PathArguments) -> Vec<TypeRef<Written>> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().filter_map(|argument| self.argument(argument)).collect()
            }
            PathArguments::None | PathArguments::Parenthesized(_) => Vec::new(),
        }
    }

    fn argument(self, argument: &GenericArgument) -> Option<TypeRef<Written>> {
        match argument {
            GenericArgument::Type(ty) => Some(self.scope.lowered(ty)),
            _ => None,
        }
    }
}
