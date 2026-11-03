use syn::{Expr, ExprField, UnOp};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(super) struct Focus;

impl Focus {
    const VIEWS: [Word<'static>; 11] = words![
        "clone", "as_ref", "as_mut", "as_str", "as_slice", "as_deref", "as_path", "borrow", "to_owned", "copied",
        "cloned"
    ];

    pub(super) fn reached(found: &Expr) -> Option<&Expr> {
        match found {
            Expr::Reference(reference) => Self::reached(&reference.expr),
            Expr::Paren(paren) => Self::reached(&paren.expr),
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => Self::reached(&unary.expr),
            Expr::MethodCall(call) if call.args.is_empty() => {
                Name::from(&call.method).matches::<Whole>(&Self::VIEWS).and_then(|_| Self::reached(&call.receiver))
            }
            Expr::Field(field) => Self::rooted(field),
            _ => None,
        }
    }

    fn rooted(field: &ExprField) -> Option<&Expr> {
        match field.base.as_ref() {
            Expr::Field(inner) => Self::rooted(inner),
            Expr::Path(path) => path.path.get_ident().map(|_| field.base.as_ref()),
            _ => None,
        }
    }
}
