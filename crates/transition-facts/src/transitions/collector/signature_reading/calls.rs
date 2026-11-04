use syn::visit::{self, Visit};
use syn::{Block, ExprCall, ExprMethodCall, Item};

use lint_core::Syntax;
use lint_layout::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Calls(Vec<Name>);

impl Calls {
    pub(super) fn made(body: &Block) -> Vec<Name> {
        let mut scan = Calls::default();
        scan.visit_block(body);
        scan.0
    }
}

impl<'ast> Visit<'ast> for Calls {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_expr_method_call(&mut self, found: &'ast ExprMethodCall) {
        visit::visit_expr_method_call(self, found);
        self.0.push(Name::from(&found.method));
    }

    fn visit_expr_call(&mut self, found: &'ast ExprCall) {
        visit::visit_expr_call(self, found);
        self.0.extend(
            Syntax::expr_path(&found.func)
                .and_then(|path| path.path.segments.last())
                .map(|last| Name::from(&last.ident)),
        );
    }
}
