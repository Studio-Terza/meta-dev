use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprCall, ExprMethodCall, Item, PatIdent, Signature, Type};

use body_lifts::{Body, BodyReading, Branches, Branching, Propagation};
use code_model::{CallOrder, Definition, Invocation, ItemPath, Mutation, Receiving, TypeRef};
use lint_core::Syntax;
use lint_layout::{Name, Whole, Word};

use crate::namespace::{Resolving, ScopeId};

use super::signature::{InScope, Written};

#[derive(Debug)]
enum LoweredReceiving {
    Owner,
    Bound(TypeRef<Written>),
    Call(CallOrder),
    Unknown,
}

#[derive(Debug)]
enum LoweredInvocation {
    Method { receiver: LoweredReceiving, name: Name },
    Path(ItemPath),
}

#[derive(Debug)]
pub(super) struct LoweredDefinition {
    shape: Body,
    branching: Branching,
    invocations: Vec<LoweredInvocation>,
}

#[derive(Debug)]
struct Invocations {
    bound: BTreeMap<Name, TypeRef<Written>>,
    made: Vec<LoweredInvocation>,
}

#[derive(Debug, Default)]
struct Rebound(BTreeSet<Name>);

impl LoweredDefinition {
    pub(super) fn of_body(scope: &InScope, signature: &Signature, body: &Block) -> LoweredDefinition {
        let rebound = Rebound::bindings_in(body);
        let bound = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter_map(|typed| Some((Name::from(&Syntax::pat_ident(&typed.pat)?.ident), &typed.ty)))
            .filter(|(name, _)| rebound.get(name).is_none())
            .map(|(name, ty)| (name, scope.lowered(ty).dereferenced().clone()))
            .collect();
        let mut scan = Invocations { bound, made: Vec::new() };
        scan.visit_block(body);
        LoweredDefinition {
            shape: BodyReading::classified(body),
            branching: Branches::scored(body, Propagation::Ambient),
            invocations: scan.made,
        }
    }

    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Definition {
        let invocations =
            self.invocations.into_iter().map(|invocation| invocation.resolved(resolving, scope)).collect();
        Definition::new(self.shape, self.branching, invocations)
    }
}

impl LoweredInvocation {
    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Invocation {
        match self {
            LoweredInvocation::Method { receiver, name } => {
                Invocation::method(receiver.resolved(resolving, scope), name)
            }
            LoweredInvocation::Path(path) => Invocation::Path(path),
        }
    }
}

impl LoweredReceiving {
    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Receiving {
        match self {
            LoweredReceiving::Owner => Receiving::Owner,
            LoweredReceiving::Bound(ty) => Receiving::Bound(Written::typed(&ty, resolving, scope)),
            LoweredReceiving::Call(order) => Receiving::Call(order),
            LoweredReceiving::Unknown => Receiving::Unknown,
        }
    }
}

impl Invocations {
    const OWNER: Word<'static> = Word::new("self");

    fn receiver(&self, found: &Expr, called: Option<CallOrder>) -> LoweredReceiving {
        match found {
            Expr::Path(path) => {
                path.path.get_ident().map_or(LoweredReceiving::Unknown, |ident| self.named(&Name::from(ident)))
            }
            Expr::MethodCall(_) => called.map_or(LoweredReceiving::Unknown, LoweredReceiving::Call),
            Expr::Call(call) => {
                Syntax::expr_path(&call.func).and(called).map_or(LoweredReceiving::Unknown, LoweredReceiving::Call)
            }
            _ => LoweredReceiving::Unknown,
        }
    }

    fn named(&self, name: &Name) -> LoweredReceiving {
        name.matches::<Whole>(&[Self::OWNER]).map_or_else(
            || self.bound.get(name).map_or(LoweredReceiving::Unknown, |ty| LoweredReceiving::Bound(ty.clone())),
            |_| LoweredReceiving::Owner,
        )
    }
}

impl<'ast> Visit<'ast> for Invocations {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_expr_method_call(&mut self, found: &'ast ExprMethodCall) {
        self.visit_expr(&found.receiver);
        let called = CallOrder::of(&self.made);
        let receiver = self.receiver(&found.receiver, called);
        found.args.iter().for_each(|argument| self.visit_expr(argument));
        self.made.push(LoweredInvocation::Method { receiver, name: Name::from(&found.method) });
    }

    fn visit_expr_call(&mut self, found: &'ast ExprCall) {
        visit::visit_expr_call(self, found);
        let written = Syntax::expr_path(&found.func)
            .map(|path| path.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect::<ItemPath>());
        self.made.extend(written.map(LoweredInvocation::Path));
    }
}

impl Rebound {
    fn bindings_in(body: &Block) -> BTreeSet<Name> {
        let mut scan = Rebound::default();
        scan.visit_block(body);
        scan.0
    }
}

impl<'ast> Visit<'ast> for Rebound {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_pat_ident(&mut self, found: &'ast PatIdent) {
        self.0.insert(Name::from(&found.ident));
        visit::visit_pat_ident(self, found);
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Mutations;

impl Mutations {
    pub(super) fn of(signature: &Signature) -> Mutation {
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
