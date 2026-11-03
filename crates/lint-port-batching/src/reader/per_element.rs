use syn::{Expr, ExprMethodCall};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct PerElement;

#[derive(Debug)]
struct Iterated;

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

    pub fn adaptor(call: &ExprMethodCall) -> Option<Word<'static>> {
        let method = Name::from(&call.method);
        let always = method.matches::<Whole>(&Self::ALWAYS);
        let shared = || method.matches::<Whole>(&Self::SHARED).filter(|_| Self::iterated(&call.receiver).is_some());
        always.or_else(shared)
    }

    fn iterated(receiver: &Expr) -> Option<Iterated> {
        match receiver {
            Expr::Range(_) => Some(Iterated),
            Expr::Paren(inner) => Self::iterated(&inner.expr),
            Expr::MethodCall(call) => Self::chained(call),
            _ => None,
        }
    }

    fn chained(call: &ExprMethodCall) -> Option<Iterated> {
        let method = Name::from(&call.method);
        let shared = || method.matches::<Whole>(&Self::SHARED).and_then(|_| Self::iterated(&call.receiver));
        method.matches::<Whole>(&Self::SOURCES).map(|_| Iterated).or_else(shared)
    }
}
