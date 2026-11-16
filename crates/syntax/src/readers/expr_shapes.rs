use syn::{Expr, ExprCall, ExprMethodCall, Ident};

#[derive(Debug)]
pub struct Constant;

impl Constant {
    #[must_use]
    pub fn composed(expr: &Expr) -> Option<&Expr> {
        Self::parts(expr).map(|()| expr)
    }

    fn parts(expr: &Expr) -> Option<()> {
        match expr {
            Expr::Lit(_) | Expr::Path(_) => Some(()),
            Expr::Paren(inner) => Self::parts(&inner.expr),
            Expr::Unary(inner) => Self::parts(&inner.expr),
            Expr::Tuple(tuple) => tuple.elems.iter().try_for_each(Self::parts),
            Expr::Array(array) => array.elems.iter().try_for_each(Self::parts),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Application;

impl Application {
    #[must_use]
    pub fn root(expr: &Expr) -> Option<&Expr> {
        match expr {
            Expr::Path(path) => path.path.get_ident().map(|_| expr),
            Expr::Reference(reference) => Self::root(&reference.expr),
            Expr::Paren(paren) => Self::root(&paren.expr),
            Expr::Field(field) => Self::root(&field.base),
            Expr::MethodCall(call) => Self::each(call.args.iter()).and_then(|_| Self::root(&call.receiver)),
            Expr::Call(call) => Self::called(call),
            _ => None,
        }
    }

    fn called(call: &ExprCall) -> Option<&Expr> {
        let arguments = Some(call).filter(|path_call| matches!(path_call.func.as_ref(), Expr::Path(_)));
        arguments.and_then(|path_call| Self::each(path_call.args.iter())).and_then(|roots| roots.into_iter().next())
    }

    fn each<'expr>(arguments: impl Iterator<Item = &'expr Expr>) -> Option<Vec<&'expr Expr>> {
        arguments.map(Self::root).collect()
    }
}

#[derive(Debug)]
pub struct Constructed<'rule, Wraps, Converts> {
    wraps: &'rule Wraps,
    converts: &'rule Converts,
}

impl<'rule, Wraps: Fn(&Expr) -> Option<()>, Converts: Fn(&Ident) -> Option<()>> Constructed<'rule, Wraps, Converts> {
    #[must_use]
    pub fn new(wraps: &'rule Wraps, converts: &'rule Converts) -> Constructed<'rule, Wraps, Converts> {
        Constructed { wraps, converts }
    }

    #[must_use]
    pub fn parts<'expr>(&self, expr: &'expr Expr) -> Vec<&'expr Expr> {
        match expr {
            Expr::Reference(reference) => self.parts(&reference.expr),
            Expr::Paren(paren) => self.parts(&paren.expr),
            Expr::MethodCall(call) => self.converted(expr, call),
            Expr::Call(call) => self.wrapped(expr, call),
            Expr::Array(array) => array.elems.iter().flat_map(|element| self.parts(element)).collect(),
            Expr::Tuple(tuple) => tuple.elems.iter().flat_map(|element| self.parts(element)).collect(),
            Expr::Struct(built) => built.fields.iter().flat_map(|field| self.parts(&field.expr)).collect(),
            _ => vec![expr],
        }
    }

    fn converted<'expr>(&self, expr: &'expr Expr, call: &'expr ExprMethodCall) -> Vec<&'expr Expr> {
        (self.converts)(&call.method).map_or_else(|| vec![expr], |()| self.parts(&call.receiver))
    }

    fn wrapped<'expr>(&self, expr: &'expr Expr, call: &'expr ExprCall) -> Vec<&'expr Expr> {
        let arguments = (self.wraps)(&call.func).map(|()| call.args.iter());
        arguments.map_or_else(|| vec![expr], |wrapped| wrapped.flat_map(|argument| self.parts(argument)).collect())
    }
}
