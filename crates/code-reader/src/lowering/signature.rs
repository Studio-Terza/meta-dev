use syn::{GenericArgument, GenericParam, Generics, PathArguments, ReturnType, Type, TypePath};

use code_model::{AliasParameter, Named, Resolved, TypeRef};
use lint_layout::{Name, Whole, Word};

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

    fn bound(held: TypeRef<Written>, path: &syn::Path) -> TypeRef<Written> {
        match path.segments.get(1) {
            Some(_) => TypeRef::Opaque,
            None => held,
        }
    }

    fn parameter(name: Name) -> TypeRef<Written> {
        TypeRef::Named(Named::new(Written::Param(name), Vec::new()))
    }
}

#[derive(Debug)]
pub(super) struct DeclaredParameter {
    name: Name,
    default: Option<TypeRef<Written>>,
}

impl DeclaredParameter {
    fn written(scope: &InScope, parameter: &GenericParam) -> Option<DeclaredParameter> {
        match parameter {
            GenericParam::Type(declared) => Some(DeclaredParameter {
                name: Name::from(&declared.ident),
                default: declared.default.as_ref().map(|ty| scope.lowered(ty)),
            }),
            GenericParam::Lifetime(_) | GenericParam::Const(_) => None,
        }
    }

    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> AliasParameter {
        AliasParameter::new(self.name, self.default.map(|ty| Written::typed(ty, resolving, scope)))
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

#[derive(Clone, Debug)]
pub(super) struct InScope {
    parameters: Vec<Name>,
    subject: TypeRef<Written>,
}

impl InScope {
    const SELF: Word<'static> = Word::new("Self");

    pub(super) fn outside() -> InScope {
        InScope { parameters: Vec::new(), subject: Written::parameter(Name::from(InScope::SELF)) }
    }

    pub(super) fn extended(&self, generics: &Generics) -> InScope {
        let declared = generics.params.iter().filter_map(TypeParameter::named);
        InScope { parameters: self.parameters.iter().cloned().chain(declared).collect(), subject: self.subject.clone() }
    }

    pub(super) fn subject(&self, subject: TypeRef<Written>) -> InScope {
        InScope { parameters: self.parameters.clone(), subject }
    }

    pub(super) fn subjectless(&self) -> InScope {
        InScope { parameters: self.parameters.clone(), subject: Written::parameter(Name::from(InScope::SELF)) }
    }

    pub(super) fn declarations(&self, generics: &Generics) -> Vec<DeclaredParameter> {
        generics.params.iter().filter_map(|parameter| DeclaredParameter::written(self, parameter)).collect()
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
        match path.segments.first().map(|first| Name::from(&first.ident)).and_then(|first| self.held(&first)) {
            Some(held) => Written::bound(held, path),
            None => TypeRef::Named(Named::new(Written::Path(UsePath::from(path)), self.arguments(path))),
        }
    }

    fn held(&self, first: &Name) -> Option<TypeRef<Written>> {
        let declared = self.parameters.iter().find(|parameter| **parameter == *first).cloned().map(Written::parameter);
        declared.or_else(|| first.matches::<Whole>(&[Self::SELF]).map(|_| self.subject.clone()))
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
