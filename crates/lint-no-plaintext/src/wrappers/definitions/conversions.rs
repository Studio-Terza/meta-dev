use syn::visit::Visit;
use syn::{ExprCall, ExprMethodCall, Ident, ItemImpl};

use lint_core::Syntax;

pub struct Conversions<'item> {
    called: Vec<&'item Ident>,
}

impl<'item> Conversions<'item> {
    const NAMES: [&'static str; 2] = ["from_str", "parse"];

    pub fn called_in(item: &'item ItemImpl) -> Option<&'item Ident> {
        let mut conversions = Conversions { called: Vec::new() };
        conversions.visit_item_impl(item);
        conversions.called.into_iter().find(|ident| Self::NAMES.iter().any(|name| ident == name))
    }
}

impl<'item> Visit<'item> for Conversions<'item> {
    fn visit_expr_call(&mut self, call: &'item ExprCall) {
        self.called.extend(
            Syntax::expr_path(&call.func).and_then(|path| path.path.segments.last()).map(|segment| &segment.ident),
        );
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'item ExprMethodCall) {
        self.called.push(&call.method);
        syn::visit::visit_expr_method_call(self, call);
    }
}
