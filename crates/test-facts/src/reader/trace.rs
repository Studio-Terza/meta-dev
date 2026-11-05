mod evaluation;
mod extent;
mod iterations;
mod locals;
mod lowering;
mod quantifiers;

use syn::Expr;

use lint_core::Syntax;

use super::scope::{Place, Scope};
use crate::claim::{Accepted, Claim};
pub(crate) use evaluation::Evaluation;
pub(crate) use extent::Extent;
pub(crate) use iterations::Iterations;
pub(crate) use locals::Locals;
pub(crate) use lowering::{Lowering, Operations};
pub(crate) use quantifiers::Quantifiers;

#[derive(Debug)]
pub(crate) struct Trace<'body, 'file> {
    scope: &'body Scope<'file>,
    locals: &'body Locals<'file>,
    place: &'body Place,
    quantifiers: &'body Quantifiers,
}

impl<'body, 'file> Trace<'body, 'file> {
    pub(crate) fn new(
        scope: &'body Scope<'file>,
        locals: &'body Locals<'file>,
        place: &'body Place,
        quantifiers: &'body Quantifiers,
    ) -> Trace<'body, 'file> {
        Trace { scope, locals, place, quantifiers }
    }

    pub(crate) fn stated(&self, subject: &Expr, accepted: Accepted) -> Vec<Claim> {
        Claim::stated(self.quantifiers.binders(), Lowering::term_of(self, subject), accepted)
    }

    pub(crate) fn scope(&self) -> &'body Scope<'file> {
        self.scope
    }

    pub(crate) fn place(&self) -> &'body Place {
        self.place
    }

    pub(crate) fn quantifiers(&self) -> &'body Quantifiers {
        self.quantifiers
    }

    pub(crate) fn bound(&self, expression: &Expr) -> Option<&'file Expr> {
        Locals::named(expression)
            .and_then(|name| self.locals.bound(&name))
            .or_else(|| Syntax::expr_path(expression).and_then(|path| self.scope.constant(&path.path, self.place)))
    }
}
