use std::collections::BTreeSet;

use syn::Expr;

use lint_layout::Name;

use super::{Constants, Locals, Trace};
use crate::check::Relation;

pub(crate) struct Tautology<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
}

impl<'trace, 'body, 'file> Tautology<'trace, 'body, 'file> {
    pub(crate) fn among(trace: &'trace Trace<'body, 'file>, sides: &[&Expr]) -> Option<Relation> {
        let tautology = Tautology { trace };
        let settled: Vec<Expr> = sides.iter().map(|side| tautology.settled(side, &mut BTreeSet::new())).collect();
        Some(settled)
            .filter(|settled| settled.len() > 1)
            .filter(|settled| Trace::distinct(&settled.iter().collect::<Vec<&Expr>>()).len() == 1)
            .map(|_| Relation::Holds)
    }

    fn settled(&self, side: &Expr, followed: &mut BTreeSet<Name>) -> Expr {
        Locals::named(side)
            .filter(|name| followed.insert(name.clone()))
            .and_then(|_| self.trace.bound(side, Constants::followed_in(side)))
            .map_or_else(|| side.clone(), |next| self.settled(next, followed))
    }
}
