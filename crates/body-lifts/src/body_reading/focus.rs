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
            Expr::Unary(unary) => Some(unary)
                .filter(|candidate| matches!(candidate.op, UnOp::Deref(_)))
                .and_then(|deref| Self::reached(&deref.expr)),
            Expr::MethodCall(call) => Some(call)
                .filter(|candidate| candidate.args.is_empty())
                .filter(|bare| Name::from(&bare.method).matches::<Whole>(&Self::VIEWS).is_some())
                .and_then(|view| Self::reached(&view.receiver)),
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
