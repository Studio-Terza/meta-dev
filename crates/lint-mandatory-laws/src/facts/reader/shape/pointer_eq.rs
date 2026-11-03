use lint_core::Syntax;
use syn::{Expr, ExprCall, Ident, ImplItemFn, ItemImpl, Stmt};

#[derive(Debug)]
pub(super) struct PointerEq;

impl PointerEq {
    const COMPARING: &str = "eq";
    const POINTER_MODULE: &str = "ptr";
    const POINTER_METHODS: [&'static str; 2] = ["ptr_eq", "addr_eq"];

    pub(super) fn whole_impl(item: &ItemImpl) -> Option<&ImplItemFn> {
        item.items.iter().filter_map(Syntax::impl_fn).find_map(Self::whole_body)
    }

    fn whole_body(function: &ImplItemFn) -> Option<&ImplItemFn> {
        Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Syntax::tail)
            .and_then(Self::address_comparison)
            .map(|_| function)
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
        Syntax::expr_path(&call.func)
            .and_then(|path| {
                Self::pointer_compare(&path.path.segments.iter().map(|segment| &segment.ident).collect::<Vec<_>>())
            })
            .map(|_| call)
    }

    fn pointer_compare<'name>(names: &[&'name Ident]) -> Option<&'name Ident> {
        let (last, leading) = names.split_last()?;
        leading
            .last()
            .filter(|module| ***module == Self::POINTER_MODULE)
            .and(Some(*last))
            .filter(|last| **last == Self::COMPARING)
            .or_else(|| Self::POINTER_METHODS.iter().find(|name| **last == **name).map(|_| *last))
    }
}
