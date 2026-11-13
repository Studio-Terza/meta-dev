use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Block, Expr, ExprClosure, ExprField, ExprForLoop, ExprLoop, ExprMethodCall, ExprPath, ExprWhile, FnArg, Item, Pat,
    Signature,
};

use code_model::{Looping, ParamIndex, Reached, Repeating, Repetition};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name, Whole, Word, words};

#[derive(Debug)]
struct Bindings {
    parameters: BTreeMap<Name, ParamIndex>,
    shadowed: BTreeSet<Name>,
}

#[derive(Debug)]
pub(super) struct Repetitions {
    bindings: Bindings,
    within: Option<Repeating>,
    found: Vec<Repetition>,
}

#[derive(Clone, Copy, Debug)]
struct SelfField;

#[derive(Clone, Copy, Debug)]
struct PerElement;

#[derive(Clone, Copy, Debug)]
struct Bound;

impl Repetitions {
    pub(super) fn of_body(signature: &Signature, body: &Block) -> Vec<Repetition> {
        let parameters = signature
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| Some((Bound::named(input)?, ParamIndex::from(index))))
            .collect();
        let bindings = Bindings { parameters, shadowed: BTreeSet::new() };
        let mut scan = Repetitions { bindings, within: None, found: Vec::new() };
        scan.visit_block(body);
        scan.found
    }

    fn within(&mut self, repeating: Option<Repeating>, visit: impl FnOnce(&mut Self)) {
        let inner = self.within.clone().or(repeating);
        let outer = mem::replace(&mut self.within, inner);
        visit(self);
        self.within = outer;
    }

    fn reached(&self, expression: &Expr) -> Option<Reached> {
        match expression {
            Expr::Path(path) => self.bindings.reached(path),
            Expr::Field(field) => SelfField::reached(field),
            _ => None,
        }
    }

    fn receiving(&mut self, call: &ExprMethodCall) {
        match self.reached(&call.receiver) {
            Some(reached) => self.found.extend(self.repetition(
                reached,
                LineNumber::from(call.receiver.span()),
                Some(Name::from(&call.method)),
            )),
            None => self.visit_expr(&call.receiver),
        }
    }

    fn repetition(&self, reached: Reached, line: LineNumber, method: Option<Name>) -> Option<Repetition> {
        self.within.clone().map(|repeating| Repetition::new(reached, line, repeating, method))
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

impl<'ast> Visit<'ast> for Repetitions {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.receiving(call);
        let adaptor = PerElement::adaptor(call).map(Repeating::Closure);
        call.args.iter().for_each(|argument| {
            let closure = Syntax::closure(argument).and(adaptor.clone());
            self.within(closure, |scan| scan.visit_expr(argument));
        });
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        let declared: BTreeSet<Name> = closure.inputs.iter().filter_map(Bound::pattern).collect();
        let added: Vec<Name> = declared.difference(&self.bindings.shadowed).cloned().collect();
        self.bindings.shadowed.extend(added.iter().cloned());
        self.visit_expr(&closure.body);
        let restored: BTreeSet<Name> = added.into_iter().collect();
        self.bindings.shadowed = self.bindings.shadowed.difference(&restored).cloned().collect();
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.visit_expr(&expr.expr);
        self.within(Some(Repeating::Loop(Looping::For)), |scan| scan.visit_block(&expr.body));
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.visit_expr(&expr.cond);
        self.within(Some(Repeating::Loop(Looping::While)), |scan| scan.visit_block(&expr.body));
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.within(Some(Repeating::Loop(Looping::Bare)), |scan| scan.visit_block(&expr.body));
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        let found = self
            .bindings
            .reached(path)
            .and_then(|reached| self.repetition(reached, LineNumber::from(path.path.span()), None));
        self.found.extend(found);
    }

    fn visit_expr_field(&mut self, field: &'ast ExprField) {
        match SelfField::reached(field) {
            Some(reached) => self.found.extend(self.repetition(reached, LineNumber::from(field.member.span()), None)),
            None => visit::visit_expr_field(self, field),
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
