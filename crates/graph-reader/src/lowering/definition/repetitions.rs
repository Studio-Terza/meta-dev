use std::collections::{BTreeMap, BTreeSet};

use syntax::Spanned;
use syntax::{
    Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprLoop, ExprMethodCall, ExprPath, ExprWhile, FnArg, Item, Past,
    Pat, PatSpine, Signature, Spine,
};

use graph::{Looping, ParamIndex, Reached, Repeating, Repetition};
use lint_core::{Extended, Fold, Syntax, Visit};
use lint_layout::{LineNumber, Name, Whole, Word, words};
use lint_sweep::{Address, Descend, Facts, Kind};

#[derive(Debug)]
pub(super) struct Repeats {
    parameters: BTreeMap<Name, ParamIndex>,
}

#[derive(Debug)]
struct Repeated {
    at: Address,
    repeating: Option<Repeating>,
}

#[derive(Debug, Default)]
struct Entries(Vec<Repeated>);

#[derive(Debug)]
struct CallScope {
    skipped: Option<Address>,
    arguments: Vec<Repeated>,
}

#[derive(Debug, Default)]
pub(super) struct Repetitions {
    within: Option<Repeating>,
    calls: Vec<CallScope>,
    loops: Vec<Repeated>,
    arguments: Entries,
    bodies: Entries,
    shadowed: BTreeSet<Name>,
    shadows: Vec<BTreeSet<Name>>,
    found: Vec<Repetition>,
}

#[derive(Clone, Copy, Debug)]
struct SelfField;

#[derive(Clone, Copy, Debug)]
struct PerElement;

#[derive(Clone, Copy, Debug)]
struct Bound;

impl Repeats {
    pub(super) fn of_signature(signature: &Signature) -> Repeats {
        let parameters = signature
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| Some((Bound::named(input)?, ParamIndex::from(index))))
            .collect();
        Repeats { parameters }
    }

    fn reached(&self, found: &Repetitions, expression: &Expr) -> Option<Reached> {
        match expression {
            Expr::Path(path) => self.bound(found, path),
            Expr::Field(field) => SelfField::reached(field),
            _ => None,
        }
    }

    fn bound(&self, found: &Repetitions, path: &ExprPath) -> Option<Reached> {
        let name = Name::from(path.path.get_ident()?);
        found.shadowed.get(&name).map_or(Some(&name), |_| None)?;
        let index = *self.parameters.get(&name)?;
        Some(Reached::Parameter { index, name })
    }
}

impl Repetitions {
    pub(super) fn found(self) -> Vec<Repetition> {
        self.found
    }

    fn repetition(&self, reached: Reached, line: LineNumber, method: Option<Name>) -> Option<Repetition> {
        self.within.clone().map(|repeating| Repetition::new(reached, line, repeating, method))
    }

    fn looped(self, body: &Block, looping: Looping) -> Visit<Repetitions> {
        let pending = Repeated { at: Address::from(body), repeating: Some(Repeating::Loop(looping)) };
        Visit::descending(Repetitions { loops: self.loops.extended([pending]), ..self })
    }

    fn flagged(self, more: Option<Repetition>, descend: Descend) -> Visit<Repetitions> {
        Visit::new(Repetitions { found: self.found.extended(more), ..self }, descend)
    }
}

impl CallScope {
    fn skipped(&self, at: Address) -> Option<Address> {
        self.skipped.filter(|skip| *skip == at)
    }

    fn argument(&self, at: Address) -> Option<Repeating> {
        self.arguments.iter().find(|argument| argument.at == at).and_then(|argument| argument.repeating.clone())
    }
}

impl Entries {
    fn entered(self, at: Address, repeating: Option<&Repeating>, within: Option<&Repeating>) -> Entries {
        Entries(self.0.extended(repeating.map(|_| Repeated { at, repeating: within.cloned() })))
    }

    fn restored(&self, at: Address, within: Option<Repeating>) -> Option<Repeating> {
        self.0.last().filter(|top| top.at == at).map_or(within, |outer| outer.repeating.clone())
    }

    fn left(self, at: Address) -> Entries {
        let mut entries = self.0;
        entries.pop_if(|top| top.at == at);
        Entries(entries)
    }
}

impl Fold for Repeats {
    type Found = Repetitions;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Item,
            Kind::Expr,
            Kind::Block,
            Kind::ExprMethodCall,
            Kind::ExprClosure,
            Kind::ExprForLoop,
            Kind::ExprWhile,
            Kind::ExprLoop,
            Kind::ExprPath,
            Kind::ExprField,
        ]
    }

    fn on_item(&self, found: Repetitions, _: &Item, _: &Facts<'_, Item>) -> Visit<Repetitions> {
        Visit::new(found, Descend::Over)
    }

    fn on_expr(&self, found: Repetitions, _: &Expr, facts: &Facts<'_, Expr>) -> Visit<Repetitions> {
        let at = facts.address();
        let call = found.calls.last();
        let skipped = call.and_then(|calling| calling.skipped(at));
        let argument = call.and_then(|calling| calling.argument(at));
        let arguments = found.arguments.entered(at, argument.as_ref(), found.within.as_ref());
        let within = found.within.or(argument);
        Visit::new(Repetitions { within, arguments, ..found }, skipped.map_or(Descend::Into, |_| Descend::Over))
    }

    fn off_expr(&self, found: Repetitions, _: &Expr, facts: &Facts<'_, Expr>) -> Repetitions {
        let at = facts.address();
        let within = found.arguments.restored(at, found.within);
        Repetitions { arguments: found.arguments.left(at), within, ..found }
    }

    fn on_block(&self, found: Repetitions, _: &Block, facts: &Facts<'_, Block>) -> Visit<Repetitions> {
        let at = facts.address();
        let mut loops = found.loops;
        let looping = loops
            .iter()
            .position(|pending| pending.at == at)
            .map(|index| loops.swap_remove(index))
            .and_then(|pending| pending.repeating);
        let bodies = found.bodies.entered(at, looping.as_ref(), found.within.as_ref());
        let within = found.within.or(looping);
        Visit::descending(Repetitions { within, loops, bodies, ..found })
    }

    fn off_block(&self, found: Repetitions, _: &Block, facts: &Facts<'_, Block>) -> Repetitions {
        let at = facts.address();
        let within = found.bodies.restored(at, found.within);
        Repetitions { bodies: found.bodies.left(at), within, ..found }
    }

    fn on_expr_method_call(
        &self,
        found: Repetitions,
        call: &ExprMethodCall,
        facts: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Repetitions> {
        let reached = self.reached(&found, &call.receiver);
        let skipped = reached.as_ref().map(|_| Address::from(call.receiver.as_ref()));
        let repeated =
            reached.and_then(|at| found.repetition(at, LineNumber::from(call.receiver.span()), facts.name().cloned()));
        let adaptor = PerElement::adaptor(call).map(Repeating::Closure);
        let arguments = call
            .args
            .iter()
            .filter(|argument| Syntax::closure(argument).is_some())
            .map(|argument| Repeated { at: Address::from(argument), repeating: adaptor.clone() })
            .collect();
        let calls = found.calls.extended([CallScope { skipped, arguments }]);
        Repetitions { calls, ..found }.flagged(repeated, Descend::Into)
    }

    fn off_expr_method_call(
        &self,
        found: Repetitions,
        _: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Repetitions {
        let mut calls = found.calls;
        calls.pop();
        Repetitions { calls, ..found }
    }

    fn on_expr_closure(
        &self,
        found: Repetitions,
        closure: &ExprClosure,
        _: &Facts<'_, ExprClosure>,
    ) -> Visit<Repetitions> {
        let declared: BTreeSet<Name> = closure.inputs.iter().filter_map(Bound::pattern).collect();
        let added: BTreeSet<Name> = declared.difference(&found.shadowed).cloned().collect();
        let shadowed = found.shadowed.extended(added.iter().cloned());
        Visit::descending(Repetitions { shadowed, shadows: found.shadows.extended([added]), ..found })
    }

    fn off_expr_closure(&self, found: Repetitions, _: &ExprClosure, _: &Facts<'_, ExprClosure>) -> Repetitions {
        let mut shadows = found.shadows;
        let restored = shadows.pop().unwrap_or_default();
        let shadowed = found.shadowed.difference(&restored).cloned().collect();
        Repetitions { shadowed, shadows, ..found }
    }

    fn on_expr_for_loop(
        &self,
        found: Repetitions,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Repetitions> {
        found.looped(&expr.body, Looping::For)
    }

    fn on_expr_while(&self, found: Repetitions, expr: &ExprWhile, _: &Facts<'_, ExprWhile>) -> Visit<Repetitions> {
        found.looped(&expr.body, Looping::While)
    }

    fn on_expr_loop(&self, found: Repetitions, expr: &ExprLoop, _: &Facts<'_, ExprLoop>) -> Visit<Repetitions> {
        found.looped(&expr.body, Looping::Bare)
    }

    fn on_expr_path(&self, found: Repetitions, path: &ExprPath, _: &Facts<'_, ExprPath>) -> Visit<Repetitions> {
        let repeated = self
            .bound(&found, path)
            .and_then(|reached| found.repetition(reached, LineNumber::from(path.path.span()), None));
        found.flagged(repeated, Descend::Over)
    }

    fn on_expr_field(&self, found: Repetitions, field: &ExprField, _: &Facts<'_, ExprField>) -> Visit<Repetitions> {
        let reached = SelfField::reached(field);
        let descend = reached.as_ref().map_or(Descend::Into, |_| Descend::Over);
        let repeated = reached.and_then(|at| found.repetition(at, LineNumber::from(field.member.span()), None));
        found.flagged(repeated, descend)
    }
}

impl Bound {
    fn named(input: &FnArg) -> Option<Name> {
        Syntax::typed(input).and_then(|typed| Self::pattern(&typed.pat))
    }

    fn pattern(pattern: &Pat) -> Option<Name> {
        let peeled = PatSpine::from(pattern).past(|node| matches!(node, Pat::Type(_)))?.node();
        Syntax::pat_ident(peeled).map(|binding| Name::from(&binding.ident))
    }
}

impl PerElement {
    const ALWAYS: [Word<'static>; 38] = words![
        "for_each",
        "try_for_each",
        "filter_map",
        "flat_map",
        "fold",
        "try_fold",
        "rfold",
        "scan",
        "take_while",
        "skip_while",
        "map_while",
        "any",
        "all",
        "find",
        "find_map",
        "position",
        "rposition",
        "partition",
        "max_by_key",
        "min_by_key",
        "max_by",
        "min_by",
        "reduce",
        "sort_by",
        "sort_by_key",
        "sort_unstable_by",
        "sort_unstable_by_key",
        "sort_by_cached_key",
        "retain",
        "retain_mut",
        "dedup_by",
        "dedup_by_key",
        "is_sorted_by",
        "is_sorted_by_key",
        "binary_search_by",
        "binary_search_by_key",
        "partition_point",
        "chunk_by",
    ];
    const SHARED: [Word<'static>; 8] = words!["map", "filter", "inspect", "copied", "cloned", "take", "zip", "by_ref"];
    const SOURCES: [Word<'static>; 39] = words![
        "filter_map",
        "flat_map",
        "scan",
        "take_while",
        "skip_while",
        "map_while",
        "chunk_by",
        "iter",
        "iter_mut",
        "into_iter",
        "chars",
        "char_indices",
        "bytes",
        "lines",
        "split",
        "split_whitespace",
        "split_terminator",
        "rsplit",
        "splitn",
        "rsplitn",
        "matches",
        "match_indices",
        "windows",
        "chunks",
        "chunks_exact",
        "keys",
        "values",
        "values_mut",
        "into_keys",
        "into_values",
        "drain",
        "enumerate",
        "rev",
        "skip",
        "step_by",
        "chain",
        "peekable",
        "flatten",
        "fuse",
    ];

    fn adaptor(call: &ExprMethodCall) -> Option<Name> {
        let method = Name::from(&call.method);
        let always = method.matches::<Whole>(&Self::ALWAYS);
        let shared = || method.matches::<Whole>(&Self::SHARED).filter(|_| Self::iterated(&call.receiver).is_some());
        always.or_else(shared).map(|_| method.clone())
    }

    fn iterated(receiver: &Expr) -> Option<&Expr> {
        let end = Spine::from(receiver).past(|node| Self::passed(node).is_some())?;
        Self::sourced(end).map(|()| receiver)
    }

    fn passed(node: &Expr) -> Option<()> {
        match node {
            Expr::Paren(_) => Some(()),
            Expr::MethodCall(call) => Name::from(&call.method).matches::<Whole>(&Self::SHARED).map(|_| ()),
            _ => None,
        }
    }

    fn sourced(end: Past<'_, Expr>) -> Option<()> {
        match end.node() {
            Expr::Range(_) => Some(()),
            Expr::MethodCall(call) => Name::from(&call.method).matches::<Whole>(&Self::SOURCES).map(|_| ()),
            _ => None,
        }
    }
}

impl SelfField {
    const OWNER: [Word<'static>; 1] = words!["self"];

    fn reached(field: &ExprField) -> Option<Reached> {
        let base = Syntax::expr_path(&field.base)?.path.get_ident().map(Name::from)?;
        base.matches::<Whole>(&Self::OWNER)?;
        Syntax::named_member(&field.member).map(|member| Reached::Field(Name::from(member)))
    }
}
