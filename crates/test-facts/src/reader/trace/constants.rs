use syn::Expr;

use lint_layout::Name;

use crate::reader::scope::Scope;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Constants {
    Followed,
    Named,
}

impl Constants {
    pub(crate) fn followed_in(expression: &Expr) -> Constants {
        match expression {
            Expr::Path(_) => Constants::Followed,
            Expr::Reference(reference) => Constants::followed_in(&reference.expr),
            Expr::Paren(inner) => Constants::followed_in(&inner.expr),
            _ => Constants::Named,
        }
    }

    pub(crate) fn looked_up<'file>(self, scope: &Scope<'file>, name: &Name) -> Option<&'file Expr> {
        match self {
            Constants::Followed => scope.constant(name),
            Constants::Named => None,
        }
    }
}
