use syntax::{Expr, Past, Spine, UnOp};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(super) struct Focus;

impl Focus {
    const VIEWS: [Word<'static>; 11] = words![
        "clone", "as_ref", "as_mut", "as_str", "as_slice", "as_deref", "as_path", "borrow", "to_owned", "copied",
        "cloned"
    ];

    pub(super) fn reached(found: &Expr) -> Option<&Expr> {
        let end = Spine::from(found).past(|node| Self::viewed(node).is_some())?.node();
        let field = Some(end).filter(|node| matches!(node, Expr::Field(_)))?;
        Self::rooted(Spine::from(field).past(|node| matches!(node, Expr::Field(_)))?)
    }

    fn rooted(root: Past<'_, Expr>) -> Option<&Expr> {
        let node = root.node();
        Some(node).filter(|found| matches!(found, Expr::Path(path) if path.path.get_ident().is_some()))
    }

    fn viewed(node: &Expr) -> Option<()> {
        match node {
            Expr::Reference(_) | Expr::Paren(_) => Some(()),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Deref(_)).then_some(()),
            Expr::MethodCall(call) => Some(call)
                .filter(|candidate| candidate.args.is_empty())
                .and_then(|bare| Name::from(&bare.method).matches::<Whole>(&Self::VIEWS))
                .map(|_| ()),
            _ => None,
        }
    }
}
