mod shaped;

use std::collections::{BTreeMap, BTreeSet};

use syntax::{Block, Expr, ExprCall, ExprField, ExprMethodCall, ExprPath, Item, MemberText, Signature, Spine, Type};

use graph::{Argument, ParamIndex, Read, Reader, Resolved, TypeRef};
use lint_core::{Extended, Fold, Syntax, Visit};
use lint_layout::{LineNumber, Name, Whole, Word};
use lint_sweep::{Address, Descend, Facts, Kind};

use crate::namespace::{Domain, Resolving, ScopeId, UsePath};

use super::super::signature::{InScope, Written};
use shaped::Shaped;

#[derive(Clone, Debug)]
enum Spelled {
    Path(UsePath),
    Generic { owner: Name, name: Name },
    Subject { subject: TypeRef<Written>, name: Name },
}

#[derive(Clone, Debug)]
struct Rooted {
    root: Name,
    fields: Vec<Name>,
}

impl Rooted {
    fn from_argument(expr: &Expr) -> Option<Rooted> {
        let passed: Vec<&Expr> = Spine::from(expr).take_while(|node| Self::passes(node).is_some()).collect();
        let root = Spine::from(expr).past(|node| Self::passes(node).is_some())?.node();
        let ident = Syntax::expr_path(root)?.path.get_ident()?;
        let fields: Vec<Name> = passed
            .iter()
            .rev()
            .filter_map(|node| Self::field(node))
            .map(|field| Name::from(MemberText::written(&field.member).as_str()))
            .collect();
        Some(Rooted { root: Name::from(ident), fields })
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

#[derive(Clone, Debug)]
enum Passed {
    Rooted(Rooted),
    Named(UsePath),
}

impl Passed {
    fn leaf(expr: &Expr) -> Option<Passed> {
        let rooted = Rooted::from_argument(expr).map(Passed::Rooted);
        rooted.or_else(|| Syntax::expr_path(expr).map(|found| Passed::Named(UsePath::from(&found.path))))
    }

    fn rooted(self) -> Option<Rooted> {
        match self {
            Passed::Rooted(rooted) => Some(rooted),
            Passed::Named(_) => None,
        }
    }

    fn all_lowered(
        passed: Vec<Shaped<Passed>>,
        parameters: &BTreeMap<Name, ParamIndex>,
        scope: &InScope,
    ) -> Vec<Shaped<Lowered>> {
        passed.into_iter().map(|shaped| shaped.mapped(&|leaf: Passed| leaf.lowered(parameters, scope))).collect()
    }

    fn lowered(self, parameters: &BTreeMap<Name, ParamIndex>, scope: &InScope) -> Shaped<Lowered> {
        match self {
            Passed::Rooted(rooted) => Lowered::rooted(rooted, parameters),
            Passed::Named(path) => Shaped::Leaf(Lowered::Named(Spelled::under(path, scope))),
        }
    }
}

#[derive(Clone, Debug)]
enum Lowered {
    Parameter { parameter: ParamIndex, fields: Vec<Name> },
    Named(Spelled),
}

impl Lowered {
    fn rooted(rooted: Rooted, parameters: &BTreeMap<Name, ParamIndex>) -> Shaped<Lowered> {
        let parameter = parameters.get(&rooted.root).copied();
        parameter.map_or(Shaped::Opaque, |found| {
            Shaped::Leaf(Lowered::Parameter { parameter: found, fields: rooted.fields })
        })
    }

    fn all_resolved(
        arguments: Vec<Shaped<Lowered>>,
        resolving: &Resolving<'_, '_, '_>,
        scope: ScopeId,
    ) -> Vec<Argument> {
        arguments.into_iter().map(|shaped| shaped.argument(&|leaf: Lowered| leaf.resolved(resolving, scope))).collect()
    }

    fn resolved(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Argument {
        match self {
            Lowered::Parameter { parameter, fields } => Argument::Parameter { parameter, fields },
            Lowered::Named(spelled) => Argument::Named(spelled.reader(resolving, scope)),
        }
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
    passed: Vec<Shaped<Passed>>,
    line: LineNumber,
}

#[derive(Debug)]
pub(super) struct Reading;

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
    arguments: Vec<Shaped<Lowered>>,
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
            let arguments = Passed::all_lowered(read.passed, &parameters, scope);
            let spelled = Spelled::under(read.path, scope);
            Some(LoweredReadAt { spelled, parameter, fields: read.rooted.fields, arguments, line: read.line })
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

    fn called(self, path: Option<&ExprPath>, call: &ExprCall) -> Visit<Reads> {
        match path {
            Some(found) => self.recorded(found, call),
            None => Visit::descending(self),
        }
    }

    fn recorded(self, called: &ExprPath, call: &ExprCall) -> Visit<Reads> {
        let line = called.path.segments.last().map_or(LineNumber::FIRST, |last| LineNumber::from(last.ident.span()));
        let path = || UsePath::from(&called.path);
        let passed: Vec<Shaped<Passed>> =
            call.args.iter().map(|argument| Shaped::folded(argument, &Passed::leaf)).collect();
        let roots = passed.iter().filter_map(Shaped::leaf).filter_map(|leaf| leaf.clone().rooted());
        let found: Vec<LoweredRead> =
            roots.map(|rooted| LoweredRead { path: path(), rooted, passed: passed.clone(), line }).collect();
        Visit::descending(Reads { found: self.found.extended(found), ..self })
    }
}

impl Spelled {
    const SUBJECT: Word<'static> = Word::new("Self");

    fn under(path: UsePath, scope: &InScope) -> Spelled {
        let single = path.qualifier().map(|parent| parent.written()).filter(|parent| parent.as_ref().len() == 1);
        let head = single.as_ref().and_then(|parent| parent.last().cloned());
        let owner = head.as_ref().and_then(|parent| scope.generic(parent).cloned());
        let last = path.written().last().cloned();
        let subject = head
            .filter(|parent| parent.matches::<Whole>(&[Self::SUBJECT]).is_some())
            .map(|_| scope.current_subject().clone());
        let generic = owner.zip(last.clone()).map(|(generic, name)| Spelled::Generic { owner: generic, name });
        let own = subject.zip(last).map(|(implementer, name)| Spelled::Subject { subject: implementer, name });
        generic.or(own).unwrap_or(Spelled::Path(path))
    }

    fn reader(self, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Reader {
        match self {
            Spelled::Path(path) => Self::path_reader(&path, resolving, scope),
            Spelled::Generic { owner, name } => Reader::Associated { owner: Resolved::Param(owner), name },
            Spelled::Subject { subject, name } => Self::subject_reader(&subject, name, resolving, scope),
        }
    }

    fn subject_reader(
        subject: &TypeRef<Written>,
        name: Name,
        resolving: &Resolving<'_, '_, '_>,
        scope: ScopeId,
    ) -> Reader {
        let typed = Written::typed(subject, resolving, scope);
        let owner = typed.named().map(|named| AsRef::<Resolved>::as_ref(named).clone());
        Reader::Associated { owner: owner.unwrap_or(Resolved::Param(Name::from(Self::SUBJECT))), name }
    }

    fn path_reader(path: &UsePath, resolving: &Resolving<'_, '_, '_>, scope: ScopeId) -> Reader {
        let name = path.written().last().cloned().unwrap_or_default();
        path.qualifier().map_or_else(
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
            .map(|read| {
                let arguments = Lowered::all_resolved(read.arguments, resolving, scope);
                Read::new(read.spelled.reader(resolving, scope), read.parameter, read.fields, read.line)
                    .passing(arguments)
            })
            .collect()
    }
}

impl Fold for Reading {
    type Found = Reads;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Item, Kind::ExprCall, Kind::ExprMethodCall]
    }

    fn on_item(&self, found: Reads, _: &Item, _: &Facts<'_, Item>) -> Visit<Reads> {
        Visit::new(found, Descend::Over)
    }

    fn on_expr_method_call(&self, found: Reads, _: &ExprMethodCall, facts: &Facts<'_, ExprMethodCall>) -> Visit<Reads> {
        Visit::descending(Reads { calls: found.calls.counted(facts.address()), ..found })
    }

    fn on_expr_call(&self, found: Reads, call: &ExprCall, facts: &Facts<'_, ExprCall>) -> Visit<Reads> {
        Reads { calls: found.calls.counted(facts.address()), ..found }.called(Syntax::expr_path(&call.func), call)
    }
}
