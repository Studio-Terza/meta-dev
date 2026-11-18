use algebra::{Monoid, Semigroup};
mod branches;
mod reads;
mod repetitions;

use std::collections::{BTreeMap, BTreeSet};

use syntax::{Block, Expr, ExprCall, ExprMethodCall, PatIdent, Signature, Type};

use body_lifts::{Body, BodyReading, Branching, Propagated};
use graph::{CallOrder, Definition, Invocation, Mutation, Receiving, Repetition, TypeRef};
use lint_core::Syntax;
use lint_layout::{Name, Whole, Word};
use lint_sweep::{Address, Gathers, Watching};

use crate::namespace::{Resolving, ScopeId};

use super::signature::{InScope, Written};
pub(super) use branches::Branches;
use reads::{Reads, ReadsLowered, SharedParameters};
use repetitions::Repetitions;

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
    propagated: Propagated,
    invocations: Vec<LoweredInvocation>,
    repetitions: Vec<Repetition>,
    reads: ReadsLowered,
}

#[derive(Debug)]
enum Pending {
    Settled(LoweredReceiving),
    CallAt(Address),
}

#[derive(Debug)]
enum PendingInvocation {
    Method { receiver: Pending, name: Name },
    Path { owner: Option<Name>, name: Name },
}

#[derive(Debug)]
pub(crate) struct Made {
    at: Address,
    invocation: Option<PendingInvocation>,
}

#[derive(Debug, Default)]
pub(crate) struct BodyPart {
    rebound: Vec<Name>,
    made: Vec<Made>,
    branches: Branches,
    reads: Reads,
}

impl Semigroup for BodyPart {
    fn combine(self, other: BodyPart) -> BodyPart {
        BodyPart {
            rebound: self.rebound.combine(other.rebound),
            made: self.made.combine(other.made),
            branches: self.branches.combine(other.branches),
            reads: self.reads.combine(other.reads),
        }
    }
}

impl Monoid for BodyPart {}

#[derive(Debug)]
pub(crate) struct BodyGroup {
    scope: InScope,
    shape: Body,
    parameters: BTreeMap<Name, TypeRef<Written>>,
    shared: SharedParameters,
    tail: Option<Address>,
    repetitions: Repetitions,
}

impl BodyPart {
    const OWNER: Word<'static> = Word::new("self");

    pub(super) fn rebinding(found: &PatIdent) -> BodyPart {
        BodyPart { rebound: vec![Name::from(&found.ident)], ..BodyPart::default() }
    }

    pub(super) fn branching(branches: Branches) -> BodyPart {
        BodyPart { branches, ..BodyPart::default() }
    }

    pub(super) fn method_entered(found: &ExprMethodCall) -> BodyPart {
        BodyPart { reads: Reads::counted(Address::from(found)), ..BodyPart::default() }
    }

    pub(super) fn call_entered(found: &ExprCall) -> BodyPart {
        BodyPart { reads: Reads::called(found, Address::from(found)), ..BodyPart::default() }
    }

    pub(super) fn method_made(found: &ExprMethodCall) -> BodyPart {
        let receiver = Self::receiver(&found.receiver);
        let invocation = PendingInvocation::Method { receiver, name: Name::from(&found.method) };
        BodyPart { made: vec![Made { at: Address::from(found), invocation: Some(invocation) }], ..BodyPart::default() }
    }

    pub(super) fn call_made(found: &ExprCall) -> BodyPart {
        let segments: Vec<Name> = Syntax::expr_path(&found.func)
            .map_or_default(|path| path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect());
        let owner = segments.iter().rev().nth(1).cloned();
        let invocation = segments.last().cloned().map(|name| PendingInvocation::Path { owner, name });
        BodyPart { made: vec![Made { at: Address::from(found), invocation }], ..BodyPart::default() }
    }

    fn receiver(found: &Expr) -> Pending {
        match found {
            Expr::Path(path) => Pending::Settled(
                path.path.get_ident().map_or(LoweredReceiving::Unknown, |ident| Self::named(Name::from(ident))),
            ),
            Expr::MethodCall(inner) => Pending::CallAt(Address::from(inner)),
            Expr::Call(call) => Syntax::expr_path(&call.func)
                .map_or(Pending::Settled(LoweredReceiving::Unknown), |_| Pending::CallAt(Address::from(call))),
            _ => Pending::Settled(LoweredReceiving::Unknown),
        }
    }

    fn named(name: Name) -> LoweredReceiving {
        name.matches::<Whole>(&[Self::OWNER]).map_or(LoweredReceiving::Named(name), |_| LoweredReceiving::Owner)
    }
}

impl Made {
    fn order(&self, made: &[Made]) -> Option<CallOrder> {
        let at = self.receiving()?;
        let completed = made.iter().position(|found| found.at == at).map_or(0, |index| index + 1);
        let earlier = made.get(..completed).unwrap_or_default();
        let counted: Vec<&Made> = earlier.iter().filter(|found| found.invocation.is_some()).collect();
        CallOrder::latest(&counted)
    }

    fn receiving(&self) -> Option<Address> {
        match &self.invocation {
            Some(PendingInvocation::Method { receiver: Pending::CallAt(at), .. }) => Some(*at),
            Some(PendingInvocation::Method { receiver: Pending::Settled(_), .. } | PendingInvocation::Path { .. })
            | None => None,
        }
    }

    fn settled(self, order: Option<CallOrder>) -> Option<LoweredInvocation> {
        self.invocation.map(|pending| pending.settled(order))
    }
}

impl PendingInvocation {
    fn settled(self, order: Option<CallOrder>) -> LoweredInvocation {
        match self {
            PendingInvocation::Method { receiver, name } => {
                LoweredInvocation::Method { receiver: receiver.settled(order), name }
            }
            PendingInvocation::Path { owner, name } => LoweredInvocation::Path { owner, name },
        }
    }
}

impl Pending {
    fn settled(self, order: Option<CallOrder>) -> LoweredReceiving {
        match self {
            Pending::Settled(receiving) => receiving,
            Pending::CallAt(_) => order.map_or(LoweredReceiving::Unknown, LoweredReceiving::Call),
        }
    }
}

impl BodyGroup {
    pub(super) fn opened(scope: &InScope, signature: &Signature, body: &Block) -> BodyGroup {
        let parameters: BTreeMap<Name, TypeRef<Written>> = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter_map(|typed| Some((Name::from(&Syntax::pat_ident(&typed.pat)?.ident), &typed.ty)))
            .map(|(name, ty)| (name, scope.lowered(ty).dereferenced().clone()))
            .collect();
        BodyGroup {
            scope: scope.clone(),
            shape: BodyReading::classified(body),
            parameters,
            shared: Reads::shared_parameters(signature),
            tail: Reads::tail_call(body),
            repetitions: Repetitions::of_signature(signature),
        }
    }

    pub(super) fn lowered(self, part: BodyPart) -> LoweredDefinition {
        let rebound: BTreeSet<Name> = part.rebound.into_iter().collect();
        let read = part.reads.settled(&self.scope, self.shared, self.tail, &rebound);
        let bound: BTreeMap<Name, TypeRef<Written>> =
            self.parameters.into_iter().filter(|(name, _)| rebound.contains(name).then_some(()).is_none()).collect();
        let orders: Vec<Option<CallOrder>> = part.made.iter().map(|made| made.order(&part.made)).collect();
        let invocations = part
            .made
            .into_iter()
            .zip(orders)
            .filter_map(|(made, order)| made.settled(order))
            .map(|invocation| invocation.bound(&bound))
            .collect();
        let branches = Branches::opened().combine(part.branches);
        LoweredDefinition {
            shape: self.shape,
            branching: branches.score(),
            propagated: branches.propagated(),
            invocations,
            repetitions: self.repetitions.found(),
            reads: read,
        }
    }
}

impl Gathers<BodyGroup> for BodyGroup {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        let members: [&mut Watching<'_>; _] = [&mut self.repetitions];
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
        Definition::new(self.shape, self.branching, self.propagated, invocations)
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
