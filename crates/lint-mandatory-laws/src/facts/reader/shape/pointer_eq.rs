use syn::{Expr, ExprCall, Ident, ImplItem, ImplItemFn, ItemImpl, Stmt};

#[derive(Debug)]
pub(super) struct PointerEq;

impl PointerEq {
    const COMPARING: &str = "eq";
    const POINTER_MODULE: &str = "ptr";
    const POINTER_METHODS: [&'static str; 2] = ["ptr_eq", "addr_eq"];

    pub(super) fn is_whole_impl(item: &ItemImpl) -> bool {
        item.items.iter().any(|member| matches!(member, ImplItem::Fn(function) if Self::is_whole_body(function)))
    }

    fn is_whole_body(function: &ImplItemFn) -> bool {
        match function.block.stmts.as_slice() {
            [Stmt::Expr(expr, None)] => Self::compares_addresses(expr),
            _ => false,
        }
    }

    fn compares_addresses(expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => Self::is_pointer_call(call),
            Expr::MethodCall(call) => Self::POINTER_METHODS.iter().any(|name| call.method == *name),
            Expr::Paren(paren) => Self::compares_addresses(&paren.expr),
            _ => false,
        }
    }

    fn is_pointer_call(call: &ExprCall) -> bool {
        match &*call.func {
            Expr::Path(path) => Self::ends_in_pointer_compare(
                &path.path.segments.iter().map(|segment| &segment.ident).collect::<Vec<_>>(),
            ),
            _ => false,
        }
    }

    fn ends_in_pointer_compare(names: &[&Ident]) -> bool {
        match names {
            [.., module, last] if **module == Self::POINTER_MODULE && **last == Self::COMPARING => true,
            [.., last] => Self::POINTER_METHODS.iter().any(|name| **last == *name),
            [] => false,
        }
    }
}
