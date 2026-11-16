use std::collections::{BTreeMap, BTreeSet};

use syntax::{Block, Expr, ExprCall, ExprField, ExprMethodCall, Item, Signature, Spine, Type};

use graph::{ParamIndex, Read, Reader, Resolved};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};
use lint_sweep::{Address, Descend, Facts, Kind, Watch};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::super::signature::InScope;

#[derive(Debug)]
enum Spelled {
    Path(UsePath),
    Generic { owner: Name, name: Name },
}

#[derive(Debug)]
struct Rooted {
    root: Name,
    fields: Vec<Name>,
}

impl Rooted {
    fn from_argument(expr: &Expr) -> Option<Rooted> {
        let passed: Vec<&Expr> = Spine::from(expr).take_while(|node| Self::passes(node).is_some()).collect();
        let root = Spine::from(expr).past(|node| Self::passes(node).is_some())?.node();
        let ident = Syntax::expr_path(root)?.path.get_ident()?;
        let fields: Option<Vec<Name>> = passed
            .iter()
            .rev()
            .filter_map(|node| Self::field(node))
            .map(|field| Syntax::named_member(&field.member).map(Name::from))
            .collect();
        Some(Rooted { root: Name::from(ident), fields: fields? })
    }

    fn passes(node: &Expr) -> Option<()> {
        match node {
            Expr::Reference(reference) => reference.mutability.map_or(Some(()), |_| None),
            Expr::Paren(_) | Expr::Field(_) => Some(()),
            _ => None,
        }
    }

    fn field(node: &Expr) -> Option<&ExprField> {
        match node {
            Expr::Field(field) => Some(field),
            _ => None,
        }
    }

    fn whole(&self) -> Option<&Rooted> {
        self.fields.first().map_or(Some(self), |_| None)
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum Calls {
    #[default]
    Zero,
    One(Address),
    Many,
}

impl Calls {
    fn counted(self, at: Address) -> Calls {
        match self {
            Calls::Zero => Calls::One(at),
            Calls::One(_) | Calls::Many => Calls::Many,
        }
    }

    fn only(self) -> Option<Address> {
        match self {
            Calls::One(at) => Some(at),
            Calls::Zero | Calls::Many => None,
        }
    }
}

#[derive(Debug)]
struct LoweredRead {
    path: UsePath,
    rooted: Rooted,
    line: LineNumber,
}

#[derive(Debug, Default)]
pub(super) struct Reads {
    found: Vec<LoweredRead>,
    calls: Calls,
}

#[derive(Debug)]
pub(super) struct SharedParameters(BTreeMap<Name, ParamIndex>);

impl SharedParameters {
    fn unbound(self, rebound: &BTreeSet<Name>) -> BTreeMap<Name, ParamIndex> {
        self.0.into_iter().filter(|(name, _)| rebound.get(name).is_none()).collect()
    }
}

#[derive(Debug, Default)]
pub(super) struct ReadsLowered(Vec<LoweredReadAt>);

#[derive(Debug)]
struct LoweredReadAt {
    spelled: Spelled,
    parameter: ParamIndex,
    fields: Vec<Name>,
    line: LineNumber,
}

impl Reads {
    pub(super) fn shared_parameters(signature: &Signature) -> SharedParameters {
        let shared = signature.inputs.iter().enumerate().filter_map(|(index, input)| {
            let typed = Syntax::typed(input)?;
            Self::shared(&typed.ty)?;
            Some((Name::from(&Syntax::pat_ident(&typed.pat)?.ident), ParamIndex::from(index)))
        });
        SharedParameters(shared.collect())
    }

    pub(super) fn settled(
        self,
        scope: &InScope,
        shared: SharedParameters,
        tail: Option<Address>,
        rebound: &BTreeSet<Name>,
    ) -> ReadsLowered {
        let parameters = shared.unbound(rebound);
        let forwarding = self.calls.only().filter(|only| tail == Some(*only));
        let kept = self.found.into_iter().filter(|read| forwarding.and(read.rooted.whole()).is_none());
        let settled = kept.filter_map(|read| {
            let parameter = *parameters.get(&read.rooted.root)?;
            let spelled = Spelled::under(read.path, scope);
            Some(LoweredReadAt { spelled, parameter, fields: read.rooted.fields, line: read.line })
        });
        ReadsLowered(settled.collect())
    }

    fn shared(ty: &Type) -> Option<&Type> {
        match ty {
            Type::Reference(reference) => reference.mutability.map_or(Some(ty), |_| None),
            _ => Some(ty),
        }
    }

    pub(super) fn tail_call(body: &Block) -> Option<Address> {
        let only = body.stmts.first().filter(|_| body.stmts.len() == 1);
        only.and_then(Syntax::tail).and_then(Syntax::expr_call).map(Address::from)
    }

    fn recorded(&mut self, path: impl Fn() -> UsePath, arguments: Vec<&Expr>, line: LineNumber) -> Descend {
        let roots = arguments.into_iter().filter_map(Rooted::from_argument);
        self.found.extend(roots.map(|rooted| LoweredRead { path: path(), rooted, line }));
        Descend::Into
    }
}

impl Spelled {
    fn under(path: UsePath, scope: &InScope) -> Spelled {
        let single = path.parent().map(|parent| parent.written()).filter(|parent| parent.as_ref().len() == 1);
        let owner = single.and_then(|parent| scope.generic(parent.last()?).cloned());
        let last = path.written().last().cloned();
        owner.zip(last).map_or(Spelled::Path(path), |(generic, name)| Spelled::Generic { owner: generic, name })
    }

    fn reader(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Reader {
        match self {
            Spelled::Path(path) => Self::path_reader(&path, resolving, scope),
            Spelled::Generic { owner, name } => Reader::Associated { owner: Resolved::Param(owner), name },
        }
    }

    fn path_reader(path: &UsePath, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Reader {
        let name = path.written().last().cloned().unwrap_or_default();
        path.parent().map_or_else(
            || {
                let valued = resolving.path(scope, path, Domain::Values);
                valued.known().map_or_else(
                    || Reader::Constructor(resolving.path(scope, path, Domain::Types)),
                    |called| Reader::Named(called.clone()),
                )
            },
            |parent| Reader::Associated { owner: resolving.path(scope, &parent, Domain::Types), name },
        )
    }
}

impl ReadsLowered {
    pub(super) fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Vec<Read> {
        self.0
            .into_iter()
            .map(|read| Read::new(read.spelled.reader(resolving, scope), read.parameter, read.fields, read.line))
            .collect()
    }
}

impl<'ast> Watch<'ast> for Reads {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ExprCall, Kind::ExprMethodCall]
    }

    fn on_item(&mut self, _: &'ast Item, _: &Facts<'_, Item>) -> Descend {
        Descend::Over
    }

    fn on_expr_method_call(&mut self, _: &'ast ExprMethodCall, facts: &Facts<'_, ExprMethodCall>) -> Descend {
        self.calls = self.calls.counted(facts.address());
        Descend::Into
    }

    fn on_expr_call(&mut self, call: &'ast ExprCall, facts: &Facts<'_, ExprCall>) -> Descend {
        self.calls = self.calls.counted(facts.address());
        Syntax::expr_path(&call.func).map_or(Descend::Into, |found| {
            let line = found.path.segments.last().map_or(LineNumber::FIRST, |last| LineNumber::from(last.ident.span()));
            self.recorded(|| UsePath::from(&found.path), call.args.iter().collect(), line)
        })
    }
}
