use syn::Expr;

#[derive(Debug)]
pub(super) struct Applied;

impl Applied {
    pub(super) fn traced(found: &Expr) -> Option<&Expr> {
        match found {
            Expr::Path(path) => path.path.get_ident().map(|_| found),
            Expr::Reference(reference) => Self::traced(&reference.expr),
            Expr::Paren(paren) => Self::traced(&paren.expr),
            Expr::Field(field) => Self::traced(&field.base),
            Expr::MethodCall(call) => Self::each(call.args.iter()).and_then(|_| Self::traced(&call.receiver)),
            Expr::Call(call) => Some(call)
                .filter(|call| matches!(call.func.as_ref(), Expr::Path(_)))
                .and_then(|call| Self::each(call.args.iter()))
                .and_then(|arguments| arguments.into_iter().next()),
            _ => None,
        }
    }

    fn each<'body>(arguments: impl Iterator<Item = &'body Expr>) -> Option<Vec<&'body Expr>> {
        arguments.map(Self::traced).collect()
    }
}
