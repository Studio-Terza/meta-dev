use std::collections::{BTreeMap, BTreeSet};

use syn::spanned::Spanned;
use syn::{
    Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprLoop, ExprMethodCall, ExprPath, ExprWhile, FnArg, Item, Pat,
    Signature,
};

use graph::{Looping, ParamIndex, Reached, Repeating, Repetition};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name, Whole, Word, words};
use lint_sweep::{Address, Descend, Kind, Watch};

#[derive(Debug)]
struct Bindings {
    parameters: BTreeMap<Name, ParamIndex>,
    shadowed: BTreeSet<Name>,
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

#[derive(Debug)]
pub(super) struct Repetitions {
    bindings: Bindings,
    within: Option<Repeating>,
    calls: Vec<CallScope>,
    loops: Vec<Repeated>,
    arguments: Entries,
    bodies: Entries,
    shadows: Vec<BTreeSet<Name>>,
    found: Vec<Repetition>,
}

#[derive(Clone, Copy, Debug)]
struct SelfField;

#[derive(Clone, Copy, Debug)]
struct PerElement;

#[derive(Clone, Copy, Debug)]
struct Bound;

impl Repetitions {
    pub(super) fn of_signature(signature: &Signature) -> Repetitions {
        let parameters = signature
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| Some((Bound::named(input)?, ParamIndex::from(index))))
            .collect();
        Repetitions {
            bindings: Bindings { parameters, shadowed: BTreeSet::new() },
            within: None,
            calls: Vec::new(),
            loops: Vec::new(),
            arguments: Entries::default(),
            bodies: Entries::default(),
            shadows: Vec::new(),
            found: Vec::new(),
        }
    }

    pub(super) fn found(self) -> Vec<Repetition> {
        self.found
    }

    fn reached(&self, expression: &Expr) -> Option<Reached> {
        match expression {
            Expr::Path(path) => self.bindings.reached(path),
            Expr::Field(field) => SelfField::reached(field),
            _ => None,
        }
    }

    fn repetition(&self, reached: Reached, line: LineNumber, method: Option<Name>) -> Option<Repetition> {
        self.within.clone().map(|repeating| Repetition::new(reached, line, repeating, method))
    }

    fn looped(&mut self, body: &Block, looping: Looping) -> Descend {
        self.loops.push(Repeated { at: Address::from(body), repeating: Some(Repeating::Loop(looping)) });
        Descend::Into
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
    fn entered(&mut self, at: Address, repeating: Option<Repeating>, within: Option<Repeating>) -> Option<Repeating> {
        let entering = repeating.map(|inner| Repeated { at, repeating: within.clone().or(Some(inner)) });
        let outer = entering.as_ref().map(|_| Repeated { at, repeating: within.clone() });
        let inner = entering.and_then(|found| found.repeating);
        self.0.extend(outer);
        inner.or(within)
    }

    fn left(&mut self, at: Address, within: Option<Repeating>) -> Option<Repeating> {
        let leaving = self.0.last().filter(|top| top.at == at).map(|top| top.at);
        leaving.and_then(|_| self.0.pop()).map_or(within, |outer| outer.repeating)
    }
}

impl<'ast> Watch<'ast> for Repetitions {
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

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        Descend::Over
    }

    fn on_expr(&mut self, expr: &'ast Expr) -> Descend {
        let at = Address::from(expr);
        let call = self.calls.last();
        let skipped = call.and_then(|calling| calling.skipped(at));
        let argument = call.and_then(|calling| calling.argument(at));
        self.within = self.arguments.entered(at, argument, self.within.take());
        skipped.map_or(Descend::Into, |_| Descend::Over)
    }

    fn off_expr(&mut self, expr: &'ast Expr) {
        self.within = self.arguments.left(Address::from(expr), self.within.take());
    }

    fn on_block(&mut self, block: &'ast Block) -> Descend {
        let at = Address::from(block);
        let looping =
            self.loops.last().filter(|pending| pending.at == at).and_then(|pending| pending.repeating.clone());
        self.within = self.bodies.entered(at, looping, self.within.take());
        Descend::Into
    }

    fn off_block(&mut self, block: &'ast Block) {
        self.within = self.bodies.left(Address::from(block), self.within.take());
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        let reached = self.reached(&call.receiver);
        let skipped = reached.as_ref().map(|_| Address::from(call.receiver.as_ref()));
        let found = reached
            .and_then(|at| self.repetition(at, LineNumber::from(call.receiver.span()), Some(Name::from(&call.method))));
        self.found.extend(found);
        let adaptor = PerElement::adaptor(call).map(Repeating::Closure);
        let arguments = call
            .args
            .iter()
            .filter(|argument| Syntax::closure(argument).is_some())
            .map(|argument| Repeated { at: Address::from(argument), repeating: adaptor.clone() })
            .collect();
        self.calls.push(CallScope { skipped, arguments });
        Descend::Into
    }

    fn off_expr_method_call(&mut self, _: &'ast ExprMethodCall) {
        self.calls.pop();
    }

    fn on_expr_closure(&mut self, closure: &'ast ExprClosure) -> Descend {
        let declared: BTreeSet<Name> = closure.inputs.iter().filter_map(Bound::pattern).collect();
        let added: BTreeSet<Name> = declared.difference(&self.bindings.shadowed).cloned().collect();
        self.bindings.shadowed.extend(added.iter().cloned());
        self.shadows.push(added);
        Descend::Into
    }

    fn off_expr_closure(&mut self, _: &'ast ExprClosure) {
        let restored = self.shadows.pop().unwrap_or_default();
        self.bindings.shadowed = self.bindings.shadowed.difference(&restored).cloned().collect();
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        self.looped(&expr.body, Looping::For)
    }

    fn off_expr_for_loop(&mut self, _: &'ast ExprForLoop) {
        self.loops.pop();
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        self.looped(&expr.body, Looping::While)
    }

    fn off_expr_while(&mut self, _: &'ast ExprWhile) {
        self.loops.pop();
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        self.looped(&expr.body, Looping::Bare)
    }

    fn off_expr_loop(&mut self, _: &'ast ExprLoop) {
        self.loops.pop();
    }

    fn on_expr_path(&mut self, path: &'ast ExprPath) -> Descend {
        let found = self
            .bindings
            .reached(path)
            .and_then(|reached| self.repetition(reached, LineNumber::from(path.path.span()), None));
        self.found.extend(found);
        Descend::Over
    }

    fn on_expr_field(&mut self, field: &'ast ExprField) -> Descend {
        let reached = SelfField::reached(field);
        let descend = reached.as_ref().map_or(Descend::Into, |_| Descend::Over);
        let found = reached.and_then(|at| self.repetition(at, LineNumber::from(field.member.span()), None));
        self.found.extend(found);
        descend
    }
}

impl Bound {
    fn named(input: &FnArg) -> Option<Name> {
        Syntax::typed(input).and_then(|typed| Self::pattern(&typed.pat))
    }

    fn pattern(pattern: &Pat) -> Option<Name> {
        match pattern {
            Pat::Ident(binding) => Some(Name::from(&binding.ident)),
            Pat::Type(typed) => Self::pattern(&typed.pat),
            _ => None,
        }
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
        match receiver {
            Expr::Range(_) => Some(receiver),
            Expr::Paren(inner) => Self::iterated(&inner.expr),
            Expr::MethodCall(call) => Self::chained(call).map(|_| receiver),
            _ => None,
        }
    }

    fn chained(call: &ExprMethodCall) -> Option<&ExprMethodCall> {
        let method = Name::from(&call.method);
        let shared =
            || method.matches::<Whole>(&Self::SHARED).and_then(|_| Self::iterated(&call.receiver)).map(|_| call);
        method.matches::<Whole>(&Self::SOURCES).map(|_| call).or_else(shared)
    }
}

impl Bindings {
    fn reached(&self, path: &ExprPath) -> Option<Reached> {
        let name = Name::from(path.path.get_ident()?);
        self.shadowed.get(&name).map_or(Some(&name), |_| None)?;
        let index = *self.parameters.get(&name)?;
        Some(Reached::Parameter { index, name })
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
