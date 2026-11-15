mod repetitions;

use std::collections::{BTreeMap, BTreeSet};

use syn::{Block, Expr, ExprCall, ExprMethodCall, Item, PatIdent, Signature, Type};

use body_lifts::{Body, BodyReading, Branching};
use graph::{CallOrder, Definition, Invocation, Mutation, Receiving, Repetition, TypeRef};
use lint_core::Syntax;
use lint_layout::{Name, Whole, Word};
use lint_sweep::{Address, Descend, Kind, Sweep, Watch};

use crate::branches::{Branches, Propagation};
use crate::namespace::{Resolving, ScopeId};

use super::signature::{InScope, Written};
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
    invocations: Vec<LoweredInvocation>,
    repetitions: Vec<Repetition>,
}

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

impl LoweredDefinition {
    pub(super) fn of_body(scope: &InScope, signature: &Signature, body: &Block) -> LoweredDefinition {
        let mut rebound = Rebound::default();
        let mut invocations = Invocations::default();
        let mut branches = Branches::new(Propagation::Ambient);
        let mut repetitions = Repetitions::of_signature(signature);
        Sweep::over_block(body, vec![&mut rebound, &mut invocations, &mut branches, &mut repetitions]);
        let parameters: BTreeMap<Name, TypeRef<Written>> = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter_map(|typed| Some((Name::from(&Syntax::pat_ident(&typed.pat)?.ident), &typed.ty)))
            .map(|(name, ty)| (name, scope.lowered(ty).dereferenced().clone()))
            .collect();
        let bound = rebound.0.iter().fold(parameters, |mut kept: BTreeMap<Name, TypeRef<Written>>, name| {
            kept.remove(name);
            kept
        });
        LoweredDefinition {
            shape: BodyReading::classified(body),
            branching: branches.score(),
            invocations: invocations.made.into_iter().map(|made| made.bound(&bound)).collect(),
            repetitions: repetitions.found(),
        }
    }

    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Definition {
        let invocations =
            self.invocations.into_iter().map(|invocation| invocation.resolved(resolving, scope)).collect();
        Definition::new(self.shape, self.branching, invocations).repeated(self.repetitions)
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
    fn marked(&mut self, latest: Option<CallOrder>) {
        self.called = latest;
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

    fn made(&mut self, made: Option<LoweredInvocation>, at: Address) -> Descend {
        self.made.extend(made);
        let latest = CallOrder::latest(&self.made);
        self.calling
            .last_mut()
            .into_iter()
            .filter(|calling| calling.receiver == Some(at))
            .for_each(|calling| calling.marked(latest));
        Descend::Into
    }

    fn receiving(receiver: &Expr) -> Option<Address> {
        match receiver {
            Expr::MethodCall(inner) => Some(Address::from(inner)),
            Expr::Call(inner) => Some(Address::from(inner)),
            _ => None,
        }
    }
}

impl<'ast> Watch<'ast> for Invocations {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ExprMethodCall, Kind::ExprCall]
    }

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        Descend::Over
    }

    fn on_expr_method_call(&mut self, found: &'ast ExprMethodCall) -> Descend {
        self.calling.push(Calling { receiver: Self::receiving(&found.receiver), called: None });
        Descend::Into
    }

    fn off_expr_method_call(&mut self, found: &'ast ExprMethodCall) {
        let called = self.calling.pop().and_then(|calling| calling.called);
        let receiver = Self::receiver(&found.receiver, called);
        self.made(Some(LoweredInvocation::Method { receiver, name: Name::from(&found.method) }), Address::from(found));
    }

    fn off_expr_call(&mut self, found: &'ast ExprCall) {
        let segments: Vec<Name> = Syntax::expr_path(&found.func)
            .map_or_default(|path| path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect());
        let owner = segments.iter().rev().nth(1).cloned();
        let made = segments.last().cloned().map(|name| LoweredInvocation::Path { owner, name });
        self.made(made, Address::from(found));
    }
}

impl<'ast> Watch<'ast> for Rebound {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::PatIdent]
    }

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        Descend::Over
    }

    fn on_pat_ident(&mut self, found: &'ast PatIdent) -> Descend {
        self.0.insert(Name::from(&found.ident));
        Descend::Into
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
