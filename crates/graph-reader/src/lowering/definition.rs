mod boundary;
mod reads;
mod repetitions;

use lint_sweep::{Fold, Folding, Visit};
use std::collections::{BTreeMap, BTreeSet};

use syntax::{Block, Expr, ExprCall, ExprMethodCall, Item, PatIdent, Signature, Type};

use body_lifts::{Body, BodyReading, Branching};
use graph::{CallOrder, Definition, Invocation, Mutation, Receiving, Repetition, TypeRef};
use lint_core::{Extended, Syntax};
use lint_layout::{Name, Whole, Word};
use lint_sweep::{Address, Descend, Facts, Gathers, Kind, Watching};

use crate::branches::{Branches, Propagation};
use crate::namespace::{Resolving, ScopeId};

use super::signature::{InScope, Written};
pub(super) use boundary::RowIndex;
use reads::{Reading, Reads, ReadsLowered, SharedParameters};
use repetitions::{Repeats, Repetitions};

#[derive(Debug)]
enum LoweredReceiving {
    Owner,
    Named(Name),
    Bound(TypeRef<Written>),
    Call(CallOrder),
    Unknown,
}

#[derive(Debug)]
enum LoweredInvocation {
    Method { receiver: LoweredReceiving, name: Name },
    Path { owner: Option<Name>, name: Name },
}

#[derive(Debug)]
pub(super) struct LoweredDefinition {
    shape: Body,
    branching: Branching,
    invocations: Vec<LoweredInvocation>,
    repetitions: Vec<Repetition>,
    reads: ReadsLowered,
}

#[derive(Debug)]
struct Invoking;

#[derive(Debug)]
struct Rebinding;

#[derive(Debug)]
struct Calling {
    receiver: Option<Address>,
    called: Option<CallOrder>,
}

#[derive(Debug, Default)]
struct Invocations {
    made: Vec<LoweredInvocation>,
    calling: Vec<Calling>,
}

#[derive(Debug, Default)]
struct Rebound(BTreeSet<Name>);

#[derive(Debug)]
pub(crate) struct BodyGroup {
    row: RowIndex,
    scope: InScope,
    shape: Body,
    parameters: BTreeMap<Name, TypeRef<Written>>,
    shared: SharedParameters,
    tail: Option<Address>,
    rebound: Folding<Rebinding>,
    invocations: Folding<Invoking>,
    branches: Folding<Branches>,
    repetitions: Folding<Repeats>,
    reads: Folding<Reading>,
}

impl BodyGroup {
    pub(super) const KINDS: &'static [Kind] = &[
        Kind::Block,
        Kind::Item,
        Kind::PatIdent,
        Kind::Expr,
        Kind::ExprCall,
        Kind::ExprMethodCall,
        Kind::ExprClosure,
        Kind::ExprForLoop,
        Kind::ExprWhile,
        Kind::ExprLoop,
        Kind::ExprPath,
        Kind::ExprField,
        Kind::ExprIf,
        Kind::ExprTry,
        Kind::ExprMatch,
        Kind::ExprBinary,
    ];

    pub(super) fn opened(row: RowIndex, scope: &InScope, signature: &Signature, body: &Block) -> BodyGroup {
        let parameters: BTreeMap<Name, TypeRef<Written>> = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter_map(|typed| Some((Name::from(&Syntax::pat_ident(&typed.pat)?.ident), &typed.ty)))
            .map(|(name, ty)| (name, scope.lowered(ty).dereferenced().clone()))
            .collect();
        BodyGroup {
            row,
            scope: scope.clone(),
            shape: BodyReading::classified(body),
            parameters,
            shared: Reads::shared_parameters(signature),
            tail: Reads::tail_call(body),
            rebound: Folding::from(Rebinding),
            invocations: Folding::from(Invoking),
            branches: Folding::from(Branches::new(Propagation::Ambient)),
            repetitions: Folding::from(Repeats::of_signature(signature)),
            reads: Folding::from(Reading),
        }
    }

    pub(super) fn row(&self) -> RowIndex {
        self.row
    }

    pub(super) fn lowered(self) -> LoweredDefinition {
        let rebound: Rebound = self.rebound.concluded();
        let invocations: Invocations = self.invocations.concluded();
        let read = self.reads.concluded::<Reads>().settled(&self.scope, self.shared, self.tail, &rebound.0);
        let bound = rebound.0.iter().fold(self.parameters, |mut kept: BTreeMap<Name, TypeRef<Written>>, name| {
            kept.remove(name);
            kept
        });
        LoweredDefinition {
            shape: self.shape,
            branching: self.branches.concluded(),
            invocations: invocations.made.into_iter().map(|made| made.bound(&bound)).collect(),
            repetitions: self.repetitions.concluded::<Repetitions>().found(),
            reads: read,
        }
    }
}

impl Gathers<BodyGroup> for BodyGroup {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        let members: [&mut Watching<'_>; _] =
            [&mut self.rebound, &mut self.invocations, &mut self.branches, &mut self.repetitions, &mut self.reads];
        members
    }

    fn gathered(self) -> BodyGroup {
        self
    }
}

impl LoweredDefinition {
    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Definition {
        let invocations =
            self.invocations.into_iter().map(|invocation| invocation.resolved(resolving, scope)).collect();
        Definition::new(self.shape, self.branching, invocations)
            .repeated(self.repetitions)
            .reading(self.reads.resolved(resolving, scope))
    }
}

impl LoweredInvocation {
    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Invocation {
        match self {
            LoweredInvocation::Method { receiver, name } => {
                Invocation::method(receiver.resolved(resolving, scope), name)
            }
            LoweredInvocation::Path { owner, name } => Invocation::Path { owner, name },
        }
    }

    fn bound(self, bound: &BTreeMap<Name, TypeRef<Written>>) -> LoweredInvocation {
        match self {
            LoweredInvocation::Method { receiver, name } => {
                LoweredInvocation::Method { receiver: receiver.bound(bound), name }
            }
            LoweredInvocation::Path { .. } => self,
        }
    }
}

impl LoweredReceiving {
    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Receiving {
        match self {
            LoweredReceiving::Owner => Receiving::Owner,
            LoweredReceiving::Bound(ty) => Receiving::Bound(Written::typed(&ty, resolving, scope)),
            LoweredReceiving::Call(order) => Receiving::Call(order),
            LoweredReceiving::Named(_) | LoweredReceiving::Unknown => Receiving::Unknown,
        }
    }

    fn bound(self, bound: &BTreeMap<Name, TypeRef<Written>>) -> LoweredReceiving {
        match self {
            LoweredReceiving::Named(name) => {
                bound.get(&name).map_or(LoweredReceiving::Unknown, |ty| LoweredReceiving::Bound(ty.clone()))
            }
            LoweredReceiving::Owner
            | LoweredReceiving::Bound(_)
            | LoweredReceiving::Call(_)
            | LoweredReceiving::Unknown => self,
        }
    }
}

impl Calling {
    fn marked(self, at: Address, latest: Option<CallOrder>) -> Calling {
        let called = self.receiver.filter(|receiver| *receiver == at).map_or(self.called, |_| latest);
        Calling { called, ..self }
    }
}

impl Invocations {
    const OWNER: Word<'static> = Word::new("self");

    fn receiver(found: &Expr, called: Option<CallOrder>) -> LoweredReceiving {
        match found {
            Expr::Path(path) => {
                path.path.get_ident().map_or(LoweredReceiving::Unknown, |ident| Self::named(Name::from(ident)))
            }
            Expr::MethodCall(_) => called.map_or(LoweredReceiving::Unknown, LoweredReceiving::Call),
            Expr::Call(call) => {
                Syntax::expr_path(&call.func).and(called).map_or(LoweredReceiving::Unknown, LoweredReceiving::Call)
            }
            _ => LoweredReceiving::Unknown,
        }
    }

    fn named(name: Name) -> LoweredReceiving {
        name.matches::<Whole>(&[Self::OWNER]).map_or(LoweredReceiving::Named(name), |_| LoweredReceiving::Owner)
    }

    fn made(self, invocation: Option<LoweredInvocation>, at: Address) -> Invocations {
        let made = self.made.extended(invocation);
        let latest = CallOrder::latest(&made);
        let mut calling = self.calling;
        let top = calling.pop().map(|open| open.marked(at, latest));
        Invocations { made, calling: calling.extended(top) }
    }

    fn receiving(receiver: &Expr) -> Option<Address> {
        match receiver {
            Expr::MethodCall(inner) => Some(Address::from(inner)),
            Expr::Call(inner) => Some(Address::from(inner)),
            _ => None,
        }
    }
}

impl Fold for Invoking {
    type Found = Invocations;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ExprMethodCall, Kind::ExprCall]
    }

    fn on_item(&self, found: Invocations, _: &Item, _: &Facts<'_, Item>) -> Visit<Invocations> {
        Visit::new(found, Descend::Over)
    }

    fn on_expr_method_call(
        &self,
        found: Invocations,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Invocations> {
        let opened = Calling { receiver: Invocations::receiving(&call.receiver), called: None };
        Visit::descending(Invocations { calling: found.calling.extended([opened]), ..found })
    }

    fn off_expr_method_call(
        &self,
        found: Invocations,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Invocations {
        let mut calling = found.calling;
        let called = calling.pop().and_then(|open| open.called);
        let receiver = Invocations::receiver(&call.receiver, called);
        Invocations { calling, ..found }
            .made(Some(LoweredInvocation::Method { receiver, name: Name::from(&call.method) }), Address::from(call))
    }

    fn off_expr_call(&self, found: Invocations, call: &ExprCall, _: &Facts<'_, ExprCall>) -> Invocations {
        let segments: Vec<Name> = Syntax::expr_path(&call.func)
            .map_or_default(|path| path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect());
        let owner = segments.iter().rev().nth(1).cloned();
        let made = segments.last().cloned().map(|name| LoweredInvocation::Path { owner, name });
        found.made(made, Address::from(call))
    }
}

impl Fold for Rebinding {
    type Found = Rebound;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::PatIdent]
    }

    fn on_item(&self, found: Rebound, _: &Item, _: &Facts<'_, Item>) -> Visit<Rebound> {
        Visit::new(found, Descend::Over)
    }

    fn on_pat_ident(&self, found: Rebound, ident: &PatIdent, _: &Facts<'_, PatIdent>) -> Visit<Rebound> {
        Visit::descending(Rebound(found.0.extended([Name::from(&ident.ident)])))
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Mutations;

impl Mutations {
    pub(super) fn signed(signature: &Signature) -> Mutation {
        let receiver = signature
            .receiver()
            .filter(|receiver| Self::mutable(&receiver.ty).is_some())
            .map(|_| Mutation::ThroughSelf);
        let parameter = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .find_map(|typed| Self::mutable(&typed.ty))
            .map(|_| Mutation::ThroughParameter);
        receiver.or(parameter).unwrap_or(Mutation::Absent)
    }

    fn mutable(found: &Type) -> Option<&Type> {
        match found {
            Type::Reference(reference) => reference.mutability.map(|_| found),
            _ => None,
        }
    }
}
