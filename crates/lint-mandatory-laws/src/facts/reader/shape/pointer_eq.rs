use syn::{Expr, ExprCall, Ident, ImplItem, ImplItemFn, ItemImpl, Stmt};

#[derive(Debug)]
pub(super) struct PointerEq;

impl PointerEq {
    const COMPARING: &str = "eq";
    const POINTER_MODULE: &str = "ptr";
    const POINTER_METHODS: [&'static str; 2] = ["ptr_eq", "addr_eq"];

    pub(super) fn whole_impl(item: &ItemImpl) -> Option<&ImplItemFn> {
        item.items.iter().find_map(|member| match member {
            ImplItem::Fn(function) => Self::whole_body(function),
            _ => None,
        })
    }

    fn whole_body(function: &ImplItemFn) -> Option<&ImplItemFn> {
        match function.block.stmts.as_slice() {
            [Stmt::Expr(expr, None)] => Self::address_comparison(expr).map(|_| function),
            _ => None,
        }
    }

    fn address_comparison(expr: &Expr) -> Option<&Expr> {
        match expr {
            Expr::Call(call) => Self::pointer_call(call).map(|_| expr),
            Expr::MethodCall(call) => Self::POINTER_METHODS.iter().find(|name| call.method == **name).map(|_| expr),
            Expr::Paren(paren) => Self::address_comparison(&paren.expr),
            _ => None,
        }
    }

    fn pointer_call(call: &ExprCall) -> Option<&ExprCall> {
        match &*call.func {
            Expr::Path(path) => {
                Self::pointer_compare(&path.path.segments.iter().map(|segment| &segment.ident).collect::<Vec<_>>())
                    .map(|_| call)
            }
            _ => None,
        }
    }

    fn pointer_compare<'name>(names: &[&'name Ident]) -> Option<&'name Ident> {
        match names {
            [.., module, last] if **module == Self::POINTER_MODULE && **last == Self::COMPARING => Some(last),
            [.., last] => Self::POINTER_METHODS.iter().find(|name| **last == **name).map(|_| *last),
            [] => None,
        }
    }
}
