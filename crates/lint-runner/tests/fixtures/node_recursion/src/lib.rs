pub struct Walk;
impl Walk {
    pub fn depth(expr: &syn::Expr) -> Depth {
        match expr {
            syn::Expr::Paren(inner) => Self::depth(&inner.expr),
            _ => Depth,
        }
    }
}
pub struct Depth;
