use syntax::{Expr, ExprField, ExprMethodCall, FnArg, Past, Pat, PatSpine, Spine};

use graph::Reached;
use lint_core::Syntax;
use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Copy, Debug)]
pub(super) struct SelfField;

#[derive(Clone, Copy, Debug)]
pub(super) struct PerElement;

#[derive(Clone, Copy, Debug)]
pub(super) struct Bound;

impl Bound {
    pub(super) fn named(input: &FnArg) -> Option<Name> {
        Syntax::typed(input).and_then(|typed| Self::pattern(&typed.pat))
    }

    pub(super) fn pattern(pattern: &Pat) -> Option<Name> {
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

    pub(super) fn adaptor(call: &ExprMethodCall) -> Option<Name> {
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

    pub(super) fn reached(field: &ExprField) -> Option<Reached> {
        let base = Syntax::expr_path(&field.base)?.path.get_ident().map(Name::from)?;
        base.matches::<Whole>(&Self::OWNER)?;
        Syntax::named_member(&field.member).map(|member| Reached::Field(Name::from(member)))
    }
}
