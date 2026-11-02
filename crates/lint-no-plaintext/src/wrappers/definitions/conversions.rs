use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMethodCall, Ident, ItemImpl};

pub struct Conversions {
    called: Vec<Ident>,
}

impl Conversions {
    const NAMES: [&'static str; 2] = ["from_str", "parse"];

    pub fn called_in(item: &ItemImpl) -> bool {
        let mut conversions = Conversions { called: Vec::new() };
        conversions.visit_item_impl(item);
        conversions.called.iter().any(|ident| Self::NAMES.iter().any(|name| ident == name))
    }
}

impl<'ast> Visit<'ast> for Conversions {
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.called.extend(match &*call.func {
            Expr::Path(path) => path.path.segments.last().map(|segment| segment.ident.clone()),
            _ => None,
        });
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.called.push(call.method.clone());
        syn::visit::visit_expr_method_call(self, call);
    }
}
