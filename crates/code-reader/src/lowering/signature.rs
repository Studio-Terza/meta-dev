use syn::spanned::Spanned;
use syn::{
    AngleBracketedGenericArguments, FnArg, GenericArgument, GenericParam, Generics, ParenthesizedGenericArguments,
    PathArguments, PathSegment, QSelf, ReturnType, Signature, Type, TypeParam, TypeParamBound, TypePath,
    WherePredicate,
};

use code_model::{
    Access, AliasParameter, Bounds, Callable, Input, Named, Projection, Referenced, Resolved, Sited, Spelling, TypeRef,
};
use lint_layout::{LineNumber, Name, Whole, Word};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

#[derive(Clone, Debug)]
enum Referent {
    Path(UsePath),
    Param(Name),
}

impl Referent {
    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Resolved {
        match self {
            Referent::Path(path) => resolving.path(scope, &path, Domain::Types),
            Referent::Param(name) => Resolved::Param(name),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Written {
    target: Referent,
    spelling: Spelling,
}

impl Written {
    pub(super) fn path(path: UsePath, line: LineNumber) -> Written {
        let spelling = Spelling::Written(path.written(), line);
        Written { target: Referent::Path(path), spelling }
    }

    pub(super) fn typed(ty: TypeRef<Written>, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> TypeRef {
        ty.mapped(&mut |written: Written| written.sited(resolving, scope))
    }

    pub(super) fn sited(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Sited {
        Sited::new(self.target.resolved(resolving, scope), self.spelling)
    }

    fn parameter(name: Name, spelling: Spelling) -> TypeRef<Written> {
        TypeRef::Named(Named::new(Written { target: Referent::Param(name), spelling }, Vec::new()))
    }

    fn respelled(self, line: LineNumber) -> Written {
        Written { target: self.target, spelling: Spelling::SelfType(line) }
    }
}

#[derive(Clone, Copy, Debug)]
struct Mutability;

impl Mutability {
    fn access(mutability: Option<syn::token::Mut>) -> Access {
        match mutability {
            Some(_) => Access::Exclusive,
            None => Access::Shared,
        }
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

#[derive(Debug)]
pub(super) enum LoweredInput {
    Receiver,
    Typed(TypeRef<Written>),
}

impl LoweredInput {
    fn lowered(scope: &InScope, input: &FnArg) -> LoweredInput {
        match input {
            FnArg::Receiver(_) => LoweredInput::Receiver,
            FnArg::Typed(typed) => LoweredInput::Typed(scope.lowered(&typed.ty)),
        }
    }

    pub(super) fn typed(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Input {
        match self {
            LoweredInput::Receiver => Input::Receiver,
            LoweredInput::Typed(ty) => Input::Typed(Written::typed(ty, resolving, scope)),
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
        InScope { parameters: Vec::new(), subject: Written::parameter(Name::from(InScope::SELF), Spelling::Implied) }
    }

    pub(super) fn extended(&self, generics: &Generics) -> InScope {
        let declared = generics.params.iter().filter_map(TypeParameter::named);
        InScope { parameters: self.parameters.iter().cloned().chain(declared).collect(), subject: self.subject.clone() }
    }

    pub(super) fn subject(&self, subject: TypeRef<Written>) -> InScope {
        InScope { parameters: self.parameters.clone(), subject }
    }

    pub(super) fn subjectless(&self) -> InScope {
        InScope::outside().with_parameters(self)
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

    pub(super) fn inputs(&self, signature: &Signature) -> Vec<LoweredInput> {
        signature.inputs.iter().map(|input| LoweredInput::lowered(self, input)).collect()
    }

    pub(super) fn constraints(&self, generics: &Generics) -> Vec<TypeRef<Written>> {
        let constraints = Constraints { scope: self };
        let parameters = generics.params.iter().flat_map(|parameter| constraints.of_parameter(parameter));
        let predicates = generics.where_clause.iter().flat_map(|clause| clause.predicates.iter());
        parameters.chain(predicates.flat_map(|predicate| constraints.of_predicate(predicate))).collect()
    }

    pub(super) fn bounded<'bound>(&self, bounds: impl Iterator<Item = &'bound TypeParamBound>) -> TypeRef<Written> {
        let bracketed = Bracketed { scope: self };
        TypeRef::Bounded(Bounds::from(bounds.filter_map(|bound| bracketed.bound(bound)).collect::<Vec<_>>()))
    }

    pub(super) fn lowered(&self, ty: &Type) -> TypeRef<Written> {
        match ty {
            Type::Path(path) => self.type_path_lowered(path),
            Type::Tuple(tuple) => TypeRef::Tuple(tuple.elems.iter().map(|element| self.lowered(element)).collect()),
            Type::Paren(inner) => self.lowered(&inner.elem),
            Type::Group(inner) => self.lowered(&inner.elem),
            Type::Reference(reference) => TypeRef::Reference(Referenced::new(
                Mutability::access(reference.mutability),
                self.lowered(&reference.elem),
            )),
            Type::Ptr(pointer) => {
                TypeRef::Pointer(Referenced::new(Mutability::access(pointer.mutability), self.lowered(&pointer.elem)))
            }
            Type::Slice(slice) => TypeRef::Slice(Box::new(self.lowered(&slice.elem))),
            Type::Array(array) => TypeRef::Array(Box::new(self.lowered(&array.elem))),
            Type::ImplTrait(bounded) => self.bounded(bounded.bounds.iter()),
            Type::TraitObject(bounded) => self.bounded(bounded.bounds.iter()),
            Type::BareFn(function) => TypeRef::Function(Callable::new(
                function.inputs.iter().map(|input| self.lowered(&input.ty)).collect(),
                self.output(&function.output).unwrap_or_else(|| TypeRef::Tuple(Vec::new())),
            )),
            _ => TypeRef::Opaque,
        }
    }

    pub(super) fn path_lowered(&self, path: &syn::Path, line: LineNumber) -> TypeRef<Written> {
        match self.held(path, line) {
            Some(held) => Tail::after_first(path).projected(held, line),
            None => Bracketed { scope: self }.named(path, line),
        }
    }

    fn with_parameters(self, scope: &InScope) -> InScope {
        InScope { parameters: scope.parameters.clone(), subject: self.subject }
    }

    fn type_path_lowered(&self, path: &TypePath) -> TypeRef<Written> {
        match &path.qself {
            Some(qself) => Qualified { scope: self, qself, path }.lowered(),
            None => self.path_lowered(&path.path, LineNumber::from(path.span())),
        }
    }

    fn held(&self, path: &syn::Path, line: LineNumber) -> Option<TypeRef<Written>> {
        let first = path.segments.first().map(|first| Name::from(&first.ident));
        let declared = first.as_ref().and_then(|first| self.parameters.iter().find(|parameter| *parameter == first));
        let parameter = declared.map(|name| {
            Written::parameter(name.clone(), Spelling::Written([name.clone()].into_iter().collect(), line))
        });
        let subject = first
            .as_ref()
            .and_then(|first| first.matches::<Whole>(&[Self::SELF]))
            .map(|_| self.subject.clone().respelled(|written| written.respelled(line)));
        parameter.or(subject)
    }
}

#[derive(Clone, Copy, Debug)]
struct Constraints<'scope> {
    scope: &'scope InScope,
}

impl Constraints<'_> {
    fn of_parameter(self, parameter: &GenericParam) -> Vec<TypeRef<Written>> {
        match parameter {
            GenericParam::Type(declared) => self.of_type_parameter(declared),
            GenericParam::Const(declared) => vec![self.scope.lowered(&declared.ty)],
            GenericParam::Lifetime(_) => Vec::new(),
        }
    }

    fn of_type_parameter(self, declared: &TypeParam) -> Vec<TypeRef<Written>> {
        let defaults = declared.default.iter().map(|ty| self.scope.lowered(ty));
        [self.scope.bounded(declared.bounds.iter())].into_iter().chain(defaults).collect()
    }

    fn of_predicate(self, predicate: &WherePredicate) -> Vec<TypeRef<Written>> {
        match predicate {
            WherePredicate::Type(bounded) => {
                vec![self.scope.lowered(&bounded.bounded_ty), self.scope.bounded(bounded.bounds.iter())]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Tail<'path> {
    item: Option<&'path PathSegment>,
    beyond: Option<&'path PathSegment>,
}

impl<'path> Tail<'path> {
    fn after_first(path: &'path syn::Path) -> Tail<'path> {
        Tail { item: path.segments.iter().nth(1), beyond: path.segments.iter().skip(1).nth(1) }
    }

    fn projected(self, held: TypeRef<Written>, line: LineNumber) -> TypeRef<Written> {
        match self.item.xor(self.beyond) {
            Some(item) => TypeRef::Projected(Box::new(Projection::new(held, None, Name::from(&item.ident), line))),
            None => self.bare(held),
        }
    }

    fn bare(self, held: TypeRef<Written>) -> TypeRef<Written> {
        match self.item {
            Some(_) => TypeRef::Opaque,
            None => held,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Qualified<'scope> {
    scope: &'scope InScope,
    qself: &'scope QSelf,
    path: &'scope TypePath,
}

impl Qualified<'_> {
    fn lowered(self) -> TypeRef<Written> {
        let segments = &self.path.path.segments;
        let item = segments.iter().nth(self.qself.position).xor(segments.iter().skip(self.qself.position).nth(1));
        let interface = self.path.path.segments.iter().take(self.qself.position).last().map(|_| {
            let written = syn::Path {
                leading_colon: self.path.path.leading_colon,
                segments: self.path.path.segments.iter().take(self.qself.position).cloned().collect(),
            };
            Bracketed { scope: self.scope }.trait_named(&written)
        });
        let line = LineNumber::from(self.path.span());
        item.map(|item| {
            TypeRef::Projected(Box::new(Projection::new(
                self.scope.lowered(&self.qself.ty),
                interface,
                Name::from(&item.ident),
                line,
            )))
        })
        .unwrap_or(TypeRef::Opaque)
    }
}

#[derive(Debug, Default)]
struct Arguments {
    arguments: Vec<TypeRef<Written>>,
    bindings: Vec<TypeRef<Written>>,
}

#[derive(Clone, Copy, Debug)]
struct Bracketed<'scope> {
    scope: &'scope InScope,
}

impl Bracketed<'_> {
    fn named(self, path: &syn::Path, line: LineNumber) -> TypeRef<Written> {
        let listed = self.last_listed(path);
        TypeRef::Named(Named::bound(Written::path(UsePath::from(path), line), listed.arguments, listed.bindings))
    }

    pub(super) fn trait_named(self, path: &syn::Path) -> Named<Written> {
        let listed = self.last_listed(path);
        Named::bound(
            Written::path(UsePath::from(path), LineNumber::from(path.span())),
            listed.arguments,
            listed.bindings,
        )
    }

    pub(super) fn bound(self, bound: &TypeParamBound) -> Option<Named<Written>> {
        match bound {
            TypeParamBound::Trait(bound) => Some(self.trait_named(&bound.path)),
            _ => None,
        }
    }

    fn last_listed(self, path: &syn::Path) -> Arguments {
        path.segments.last().map(|last| self.listed(&last.arguments)).unwrap_or_default()
    }

    fn listed(self, arguments: &PathArguments) -> Arguments {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => Angled { scope: self.scope }.listed(bracketed),
            PathArguments::Parenthesized(sugared) => Angled { scope: self.scope }.sugared(sugared),
            PathArguments::None => Arguments::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Angled<'scope> {
    scope: &'scope InScope,
}

impl Angled<'_> {
    fn listed(self, bracketed: &AngleBracketedGenericArguments) -> Arguments {
        Arguments {
            arguments: bracketed.args.iter().filter_map(|argument| self.argument(argument)).collect(),
            bindings: bracketed
                .args
                .iter()
                .filter_map(|argument| Associated { scope: self.scope }.bound(argument))
                .collect(),
        }
    }

    fn sugared(self, sugared: &ParenthesizedGenericArguments) -> Arguments {
        Arguments {
            arguments: sugared.inputs.iter().map(|input| self.scope.lowered(input)).collect(),
            bindings: self.scope.output(&sugared.output).into_iter().collect(),
        }
    }

    fn argument(self, argument: &GenericArgument) -> Option<TypeRef<Written>> {
        match argument {
            GenericArgument::Type(ty) => Some(self.scope.lowered(ty)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Associated<'scope> {
    scope: &'scope InScope,
}

impl Associated<'_> {
    fn bound(self, argument: &GenericArgument) -> Option<TypeRef<Written>> {
        match argument {
            GenericArgument::AssocType(bound) => Some(self.scope.lowered(&bound.ty)),
            GenericArgument::Constraint(constrained) => Some(self.scope.bounded(constrained.bounds.iter())),
            _ => None,
        }
    }
}
