use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    AngleBracketedGenericArguments, Expr, FnArg, GenericArgument, GenericParam, Generics,
    ParenthesizedGenericArguments, PathArguments, QSelf, ReturnType, Signature, Type, TypeParam, TypeParamBound,
    TypePath, WherePredicate,
};

use code_model::{AliasParameter, Arrayed, Callable, Input, Named, Projection, Resolved, Sited, Spelling, TypeRef};
use lint_layout::{Name, Position, Whole, Word};

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
    pub(super) fn path(path: UsePath, position: Position) -> Written {
        let spelling = Spelling::Written(path.written(), position);
        Written { target: Referent::Path(path), spelling }
    }

    pub(super) fn typed(ty: &TypeRef<Written>, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> TypeRef {
        ty.mapped(&mut |written: &Written| written.sited(resolving, scope))
    }

    fn sited(&self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Sited {
        Sited::new(self.target.clone().resolved(resolving, scope), self.spelling.clone())
    }

    fn parameter(name: Name, spelling: Spelling) -> TypeRef<Written> {
        TypeRef::Named(Named::new(Written { target: Referent::Param(name), spelling }, Vec::new()))
    }

    fn implied(self) -> Written {
        Written { target: self.target, spelling: Spelling::Implied }
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
        AliasParameter::new(self.name, self.default.map(|ty| Written::typed(&ty, resolving, scope)))
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
            LoweredInput::Typed(ty) => Input::Typed(Written::typed(&ty, resolving, scope)),
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

    pub(super) fn lowered(&self, ty: &Type) -> TypeRef<Written> {
        match ty {
            Type::Path(path) => self.type_path_lowered(path),
            Type::Tuple(tuple) => TypeRef::Tuple(tuple.elems.iter().map(|element| self.lowered(element)).collect()),
            Type::Paren(inner) => self.lowered(&inner.elem),
            Type::Reference(reference) => TypeRef::Reference(Box::new(self.lowered(&reference.elem))),
            Type::Ptr(pointer) => TypeRef::Pointer(Box::new(self.lowered(&pointer.elem))),
            Type::Slice(slice) => TypeRef::Slice(Box::new(self.lowered(&slice.elem))),
            Type::Array(array) => {
                TypeRef::Array(Arrayed::new(self.lowered(&array.elem), Measured::types(self, &array.len)))
            }
            Type::ImplTrait(bounded) => Bounding { scope: self }.lowered(bounded.bounds.iter()),
            Type::TraitObject(bounded) => Bounding { scope: self }.lowered(bounded.bounds.iter()),
            Type::BareFn(function) => TypeRef::Function(Callable::new(
                function.inputs.iter().map(|input| self.lowered(&input.ty)).collect(),
                self.output(&function.output).unwrap_or_else(|| TypeRef::Tuple(Vec::new())),
            )),
            _ => TypeRef::Opaque,
        }
    }

    fn path_lowered(&self, path: &syn::Path, position: Position) -> TypeRef<Written> {
        match self.held(path, position) {
            Some(held) => Projecting { scope: self }.held(held, path),
            None => Bracketed { scope: self }.named(path, position),
        }
    }

    fn with_parameters(self, scope: &InScope) -> InScope {
        InScope { parameters: scope.parameters.clone(), subject: self.subject }
    }

    fn type_path_lowered(&self, path: &TypePath) -> TypeRef<Written> {
        match &path.qself {
            Some(qself) => Projecting { scope: self }.qualified(qself, &path.path),
            None => self.path_lowered(&path.path, Position::from(path.span())),
        }
    }

    fn held(&self, path: &syn::Path, position: Position) -> Option<TypeRef<Written>> {
        let first = path.segments.first().map(|first| Name::from(&first.ident));
        let declared = first.as_ref().and_then(|head| self.parameters.iter().find(|parameter| *parameter == head));
        let parameter = declared.map(|name| {
            Written::parameter(name.clone(), Spelling::Written([name.clone()].into_iter().collect(), position))
        });
        let subject = first
            .as_ref()
            .and_then(|head| head.matches::<Whole>(&[Self::SELF]))
            .map(|_| self.subject.clone().respelled(Written::implied));
        parameter.or(subject)
    }
}

#[derive(Debug)]
struct Measured<'scope> {
    scope: &'scope InScope,
    found: Vec<TypeRef<Written>>,
}

impl Measured<'_> {
    fn types(scope: &InScope, length: &Expr) -> Vec<TypeRef<Written>> {
        let mut measured = Measured { scope, found: Vec::new() };
        measured.visit_expr(length);
        measured.found
    }
}

impl<'ast> Visit<'ast> for Measured<'_> {
    fn visit_type(&mut self, ty: &'ast Type) {
        self.found.push(self.scope.lowered(ty));
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
        let bounded = declared.bounds.first().map(|_| Bounding { scope: self.scope }.lowered(declared.bounds.iter()));
        let defaults = declared.default.iter().map(|ty| self.scope.lowered(ty));
        bounded.into_iter().chain(defaults).collect()
    }

    fn of_predicate(self, predicate: &WherePredicate) -> Vec<TypeRef<Written>> {
        match predicate {
            WherePredicate::Type(bounded) => {
                vec![
                    self.scope.lowered(&bounded.bounded_ty),
                    Bounding { scope: self.scope }.lowered(bounded.bounds.iter()),
                ]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct Arguments {
    arguments: Vec<TypeRef<Written>>,
    bindings: Vec<TypeRef<Written>>,
}

impl Arguments {
    fn types(self) -> impl Iterator<Item = TypeRef<Written>> {
        self.arguments.into_iter().chain(self.bindings)
    }
}

#[derive(Clone, Copy, Debug)]
struct Bracketed<'scope> {
    scope: &'scope InScope,
}

impl Bracketed<'_> {
    fn named(self, path: &syn::Path, position: Position) -> TypeRef<Written> {
        let listed = self.last_listed(path);
        TypeRef::Named(Named::bound(Written::path(UsePath::from(path), position), listed.arguments, listed.bindings))
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
struct Projecting<'scope> {
    scope: &'scope InScope,
}

impl Projecting<'_> {
    fn held(self, held: TypeRef<Written>, path: &syn::Path) -> TypeRef<Written> {
        match path.segments.get(1) {
            Some(_) => TypeRef::Projected(Projection::new(held, self.carried(path))),
            None => held,
        }
    }

    fn qualified(self, qself: &QSelf, path: &syn::Path) -> TypeRef<Written> {
        TypeRef::Projected(Projection::new(self.scope.lowered(&qself.ty), self.carried(path)))
    }

    fn carried(self, path: &syn::Path) -> Vec<TypeRef<Written>> {
        let bracketed = Bracketed { scope: self.scope };
        path.segments.iter().flat_map(|segment| bracketed.listed(&segment.arguments).types()).collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Bounding<'scope> {
    scope: &'scope InScope,
}

impl Bounding<'_> {
    fn lowered<'bound>(self, bounds: impl Iterator<Item = &'bound TypeParamBound>) -> TypeRef<Written> {
        TypeRef::Bounded(bounds.filter_map(|bound| self.bound(bound)).collect())
    }

    fn bound(self, bound: &TypeParamBound) -> Option<TypeRef<Written>> {
        match bound {
            TypeParamBound::Trait(trait_bound) => {
                Some(Bracketed { scope: self.scope }.named(&trait_bound.path, Position::from(trait_bound.path.span())))
            }
            _ => None,
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
                .flat_map(|argument| Associated { scope: self.scope }.bound(argument))
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
    fn bound(self, argument: &GenericArgument) -> Vec<TypeRef<Written>> {
        match argument {
            GenericArgument::AssocType(bound) => vec![self.scope.lowered(&bound.ty)],
            GenericArgument::Constraint(constraint) => {
                vec![Bounding { scope: self.scope }.lowered(constraint.bounds.iter())]
            }
            GenericArgument::Const(expression) => Measured::types(self.scope, expression),
            _ => Vec::new(),
        }
    }
}
