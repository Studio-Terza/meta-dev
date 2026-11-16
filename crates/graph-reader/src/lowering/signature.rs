mod bounds;

use syntax::{
    AngleBracketedGenericArguments, Expr, FnArg, GenericArgument, GenericParam, Generics, Ident,
    ParenthesizedGenericArguments, PathArguments, PredicateType, QSelf, ReturnType, Signature, TraitBoundModifier,
    Type, TypeParam, TypeParamBound, TypePath, WherePredicate,
};
use syntax::{Fold, Spanned, TypeTree};

use graph::{
    AliasParameter, Arrayed, Callable, Input, Named, Parameter, Projection, Receiver, Resolved, Sited, Spelling,
    TypeRef,
};
use lint_core::Syntax;
use lint_layout::{Name, Position, Whole, Word};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};
pub(super) use bounds::ParamBounds;

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

#[derive(Clone, Copy, Debug)]
pub struct ReceiverSyntax<'syn>(&'syn syntax::Receiver);

impl<'syn> From<&'syn syntax::Receiver> for ReceiverSyntax<'syn> {
    fn from(written: &'syn syntax::Receiver) -> ReceiverSyntax<'syn> {
        ReceiverSyntax(written)
    }
}

impl From<ReceiverSyntax<'_>> for Receiver {
    fn from(syntax: ReceiverSyntax<'_>) -> Receiver {
        let written = syntax.0;
        written
            .reference
            .as_ref()
            .map_or(Receiver::Owned, |_| written.mutability.map_or(Receiver::Shared, |_| Receiver::Mutable))
    }
}

#[derive(Debug)]
pub(super) enum LoweredInput {
    Receiver(Receiver),
    Typed(TypeRef<Written>),
}

impl LoweredInput {
    fn lowered(scope: &InScope, input: &FnArg) -> LoweredInput {
        match input {
            FnArg::Receiver(receiver) => LoweredInput::Receiver(Receiver::from(ReceiverSyntax::from(receiver))),
            FnArg::Typed(typed) => LoweredInput::Typed(scope.lowered(&typed.ty)),
        }
    }

    pub(super) fn typed(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Input {
        match self {
            LoweredInput::Receiver(receiver) => Input::Receiver(receiver),
            LoweredInput::Typed(ty) => Input::Typed(Written::typed(&ty, resolving, scope)),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct InScope {
    parameters: Vec<Name>,
    subject: TypeRef<Written>,
    bounds: Vec<ParamBounds>,
}

impl InScope {
    const SELF: Word<'static> = Word::new("Self");

    pub(super) fn outside() -> InScope {
        InScope {
            parameters: Vec::new(),
            subject: Written::parameter(Name::from(InScope::SELF), Spelling::Implied),
            bounds: Vec::new(),
        }
    }

    pub(super) fn extended(&self, generics: &Generics) -> InScope {
        let declared = generics.params.iter().filter_map(TypeParameter::named);
        InScope {
            parameters: self.parameters.iter().cloned().chain(declared).collect(),
            subject: self.subject.clone(),
            bounds: self.bounds.iter().cloned().chain(ParamBounds::declared(generics)).collect(),
        }
    }

    pub(super) fn subject(&self, subject: TypeRef<Written>) -> InScope {
        InScope { parameters: self.parameters.clone(), subject, bounds: self.bounds.clone() }
    }

    pub(super) fn generic(&self, name: &Name) -> Option<&Name> {
        self.parameters.iter().find(|parameter| *parameter == name)
    }

    pub(super) fn traits_of(&self, param: &Name) -> Vec<&UsePath> {
        self.bounds.iter().filter(|bound| bound.param() == param).flat_map(ParamBounds::traits).collect()
    }

    pub(super) fn current_subject(&self) -> &TypeRef<Written> {
        &self.subject
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

    pub(super) fn bounded<'bound>(&self, bounds: impl Iterator<Item = &'bound TypeParamBound>) -> TypeRef<Written> {
        let listed: Vec<&TypeParamBound> = bounds.collect();
        let fold = self.fold(TypeTree::bounds(&listed));
        Bounding { lowering: Lowering { scope: self, fold: &fold } }.lowered(listed.into_iter())
    }

    pub(super) fn constraints(&self, generics: &Generics) -> Vec<TypeRef<Written>> {
        let fold = self.fold(TypeTree::from(generics));
        let constraints = Constraints { lowering: Lowering { scope: self, fold: &fold } };
        let parameters = generics.params.iter().flat_map(|parameter| constraints.of_parameter(parameter));
        let predicates = generics.where_clause.iter().flat_map(|clause| clause.predicates.iter());
        parameters.chain(predicates.flat_map(|predicate| constraints.of_predicate(predicate))).collect()
    }

    pub(super) fn lowered(&self, ty: &Type) -> TypeRef<Written> {
        let fold = self.fold(TypeTree::from(ty));
        Lowering { scope: self, fold: &fold }.child(ty)
    }

    fn fold<'syn>(&self, tree: TypeTree<'syn>) -> Fold<'syn, Type, TypeRef<Written>> {
        Fold::over(tree, |fold, ty| Lowering { scope: self, fold }.lowered(ty).unwrap_or(TypeRef::Opaque))
    }

    fn with_parameters(self, scope: &InScope) -> InScope {
        InScope { parameters: scope.parameters.clone(), subject: self.subject, bounds: scope.bounds.clone() }
    }

    fn held(&self, path: &syntax::Path, position: Position) -> Option<TypeRef<Written>> {
        let first = path.segments.first().map(|first| Name::from(&first.ident));
        let declared = first.as_ref().and_then(|head| self.generic(head));
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

#[derive(Clone, Copy, Debug)]
struct Lowering<'low> {
    scope: &'low InScope,
    fold: &'low Fold<'low, Type, TypeRef<Written>>,
}

impl Lowering<'_> {
    fn child(self, ty: &Type) -> TypeRef<Written> {
        self.fold.take(ty).unwrap_or(TypeRef::Opaque)
    }

    fn output(self, output: &ReturnType) -> Option<TypeRef<Written>> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(self.child(ty)),
        }
    }

    fn lowered(self, ty: &Type) -> Option<TypeRef<Written>> {
        match ty {
            Type::Path(path) => Some(self.type_path_lowered(path)),
            Type::Tuple(tuple) => Some(TypeRef::Tuple(tuple.elems.iter().map(|element| self.child(element)).collect())),
            Type::Paren(inner) => Some(self.child(&inner.elem)),
            Type::Reference(reference) => Some(TypeRef::Reference(Box::new(self.child(&reference.elem)))),
            Type::Ptr(pointer) => Some(TypeRef::Pointer(Box::new(self.child(&pointer.elem)))),
            Type::Slice(slice) => Some(TypeRef::Slice(Box::new(self.child(&slice.elem)))),
            Type::Array(array) => {
                Some(TypeRef::Array(Arrayed::new(self.child(&array.elem), Measured::types(self, &array.len))))
            }
            Type::ImplTrait(bounded) => Some(Bounding { lowering: self }.lowered(bounded.bounds.iter())),
            Type::TraitObject(bounded) => Some(Bounding { lowering: self }.lowered(bounded.bounds.iter())),
            Type::BareFn(function) => Some(TypeRef::Function(Callable::new(
                function.inputs.iter().map(|input| self.child(&input.ty)).collect(),
                self.output(&function.output).unwrap_or_else(|| TypeRef::Tuple(Vec::new())),
            ))),
            _ => None,
        }
    }

    fn type_path_lowered(self, path: &TypePath) -> TypeRef<Written> {
        match &path.qself {
            Some(qself) => Projecting { lowering: self }.qualified(qself, &path.path),
            None => Bracketed { lowering: self }.unqualified(path),
        }
    }
}

#[derive(Debug)]
struct Measured;

impl Measured {
    fn types(lowering: Lowering<'_>, length: &Expr) -> Vec<TypeRef<Written>> {
        TypeTree::outermost(length).into_iter().map(|ty| lowering.child(ty)).collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Constraints<'low> {
    lowering: Lowering<'low>,
}

impl Constraints<'_> {
    fn of_parameter(self, parameter: &GenericParam) -> Vec<TypeRef<Written>> {
        match parameter {
            GenericParam::Type(declared) => self.of_type_parameter(declared),
            GenericParam::Const(declared) => vec![self.lowering.child(&declared.ty)],
            GenericParam::Lifetime(_) => Vec::new(),
        }
    }

    fn of_type_parameter(self, declared: &TypeParam) -> Vec<TypeRef<Written>> {
        let bounded =
            declared.bounds.first().map(|_| Bounding { lowering: self.lowering }.lowered(declared.bounds.iter()));
        let defaults = declared.default.iter().map(|ty| self.lowering.child(ty));
        bounded.into_iter().chain(defaults).collect()
    }

    fn of_predicate(self, predicate: &WherePredicate) -> Vec<TypeRef<Written>> {
        match predicate {
            WherePredicate::Type(bounded) => {
                vec![
                    self.lowering.child(&bounded.bounded_ty),
                    Bounding { lowering: self.lowering }.lowered(bounded.bounds.iter()),
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
struct Bracketed<'low> {
    lowering: Lowering<'low>,
}

impl Bracketed<'_> {
    fn unqualified(self, path: &TypePath) -> TypeRef<Written> {
        let position = Position::from(path.span());
        let held = self.lowering.scope.held(&path.path, position);
        held.map_or_else(
            || self.named(&path.path, position),
            |found| Projecting { lowering: self.lowering }.held(found, &path.path),
        )
    }

    fn named(self, path: &syntax::Path, position: Position) -> TypeRef<Written> {
        let listed = self.last_listed(path);
        TypeRef::Named(Named::bound(Written::path(UsePath::from(path), position), listed.arguments, listed.bindings))
    }

    fn last_listed(self, path: &syntax::Path) -> Arguments {
        path.segments.last().map_or_default(|last| self.listed(&last.arguments))
    }

    fn listed(self, arguments: &PathArguments) -> Arguments {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => Angled { lowering: self.lowering }.listed(bracketed),
            PathArguments::Parenthesized(sugared) => Angled { lowering: self.lowering }.sugared(sugared),
            PathArguments::None => Arguments::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Projecting<'low> {
    lowering: Lowering<'low>,
}

impl Projecting<'_> {
    fn held(self, held: TypeRef<Written>, path: &syntax::Path) -> TypeRef<Written> {
        match path.segments.get(1) {
            Some(_) => TypeRef::Projected(Projection::new(held, self.carried(path), Self::associated(path))),
            None => held,
        }
    }

    fn qualified(self, qself: &QSelf, path: &syntax::Path) -> TypeRef<Written> {
        TypeRef::Projected(Projection::new(self.lowering.child(&qself.ty), self.carried(path), Self::associated(path)))
    }

    fn associated(path: &syntax::Path) -> Option<Name> {
        path.segments.last().map(|last| Name::from(&last.ident))
    }

    fn carried(self, path: &syntax::Path) -> Vec<TypeRef<Written>> {
        let bracketed = Bracketed { lowering: self.lowering };
        path.segments.iter().flat_map(|segment| bracketed.listed(&segment.arguments).types()).collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Bounding<'low> {
    lowering: Lowering<'low>,
}

impl Bounding<'_> {
    fn lowered<'bound>(self, bounds: impl Iterator<Item = &'bound TypeParamBound>) -> TypeRef<Written> {
        TypeRef::Bounded(bounds.filter_map(|bound| self.bound(bound)).collect())
    }

    fn bound(self, bound: &TypeParamBound) -> Option<TypeRef<Written>> {
        match bound {
            TypeParamBound::Trait(trait_bound) => Some(
                Bracketed { lowering: self.lowering }.named(&trait_bound.path, Position::from(trait_bound.path.span())),
            ),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Angled<'low> {
    lowering: Lowering<'low>,
}

impl Angled<'_> {
    fn listed(self, bracketed: &AngleBracketedGenericArguments) -> Arguments {
        Arguments {
            arguments: bracketed.args.iter().filter_map(|argument| self.argument(argument)).collect(),
            bindings: bracketed
                .args
                .iter()
                .flat_map(|argument| Associated { lowering: self.lowering }.bound(argument))
                .collect(),
        }
    }

    fn sugared(self, sugared: &ParenthesizedGenericArguments) -> Arguments {
        Arguments {
            arguments: sugared.inputs.iter().map(|input| self.lowering.child(input)).collect(),
            bindings: self.lowering.output(&sugared.output).into_iter().collect(),
        }
    }

    fn argument(self, argument: &GenericArgument) -> Option<TypeRef<Written>> {
        match argument {
            GenericArgument::Type(ty) => Some(self.lowering.child(ty)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Associated<'low> {
    lowering: Lowering<'low>,
}

impl Associated<'_> {
    fn bound(self, argument: &GenericArgument) -> Vec<TypeRef<Written>> {
        match argument {
            GenericArgument::AssocType(bound) => vec![self.lowering.child(&bound.ty)],
            GenericArgument::Constraint(constraint) => {
                vec![Bounding { lowering: self.lowering }.lowered(constraint.bounds.iter())]
            }
            GenericArgument::Const(expression) => Measured::types(self.lowering, expression),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub(super) struct LoweredParameter {
    name: Name,
    bounds: Vec<TypeRef<Written>>,
}

impl LoweredParameter {
    pub(super) fn declared(scope: &InScope, generics: &Generics) -> Vec<LoweredParameter> {
        let fold = scope.fold(TypeTree::from(generics));
        let predicates: Vec<&PredicateType> = generics
            .where_clause
            .iter()
            .flat_map(|clause| clause.predicates.iter())
            .filter_map(Requirement::typed)
            .collect();
        generics
            .type_params()
            .map(|declared| {
                let required = predicates
                    .iter()
                    .filter_map(|typed| Requirement::bounding(typed, &declared.ident))
                    .flat_map(|typed| typed.bounds.iter());
                let bounding = Bounding { lowering: Lowering { scope, fold: &fold } };
                LoweredParameter {
                    name: Name::from(&declared.ident),
                    bounds: declared
                        .bounds
                        .iter()
                        .chain(required)
                        .filter_map(Requirement::plain)
                        .filter_map(|bound| bounding.bound(bound))
                        .collect(),
                }
            })
            .collect()
    }

    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Parameter {
        Parameter::new(self.name, self.bounds.iter().map(|ty| Written::typed(ty, resolving, scope)).collect())
    }
}

#[derive(Clone, Copy, Debug)]
struct Requirement;

impl Requirement {
    fn plain(bound: &TypeParamBound) -> Option<&TypeParamBound> {
        match bound {
            TypeParamBound::Trait(traited) => Requirement::unrelaxed(traited.modifier).map(|()| bound),
            _ => Some(bound),
        }
    }

    fn unrelaxed(modifier: TraitBoundModifier) -> Option<()> {
        match modifier {
            TraitBoundModifier::None => Some(()),
            TraitBoundModifier::Maybe(_) => None,
        }
    }

    fn typed(predicate: &WherePredicate) -> Option<&PredicateType> {
        match predicate {
            WherePredicate::Type(typed) => Some(typed),
            _ => None,
        }
    }

    fn bounding<'predicate>(typed: &'predicate PredicateType, name: &Ident) -> Option<&'predicate PredicateType> {
        Syntax::type_path(&typed.bounded_ty)
            .and_then(|path| path.path.get_ident())
            .filter(|ident| *ident == name)
            .map(|_| typed)
    }
}
