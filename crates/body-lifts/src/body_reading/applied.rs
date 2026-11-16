use syntax::{Expr, ExprEdge, ExprReached, ExprTree};

#[derive(Debug)]
pub(super) struct Applied;

impl Applied {
    pub(super) fn traced(found: &Expr) -> Option<&Expr> {
        let reached = ExprTree::reached(found, &|_, edge| {
            matches!(
                edge,
                ExprEdge::Inner
                    | ExprEdge::Base
                    | ExprEdge::Receiver
                    | ExprEdge::Argument(_)
                    | ExprEdge::MethodArgument(_)
            )
        });
        reached.iter().try_for_each(|node| Self::allowed(node.expr()))?;
        let chain = ExprTree::reached(found, &|_, edge| {
            matches!(edge, ExprEdge::Inner | ExprEdge::Base | ExprEdge::Receiver | ExprEdge::Argument(0))
        });
        chain.last().copied().map(ExprReached::expr).filter(|leaf| matches!(leaf, Expr::Path(_)))
    }

    fn allowed(node: &Expr) -> Option<()> {
        match node {
            Expr::Path(path) => path.path.get_ident().map(|_| ()),
            Expr::Reference(_) | Expr::Paren(_) | Expr::Field(_) | Expr::MethodCall(_) => Some(()),
            Expr::Call(call) => call.args.first().filter(|_| matches!(call.func.as_ref(), Expr::Path(_))).map(|_| ()),
            _ => None,
        }
    }
}
