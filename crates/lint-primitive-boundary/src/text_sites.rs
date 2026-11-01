use std::path::Path;

use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall, Ident, ItemImpl, ItemMod};

use super::finding::Finding;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole};

#[derive(Default)]
pub struct TextSites {
    doors: Vec<Ident>,
    found: Vec<Ident>,
}

impl TextSites {
    const DOOR_TRAITS: [&str; 3] = ["FromStr", "TryFrom", "Display"];
    const INSPECTING: [&str; 8] = [
        "split_whitespace",
        "split_once",
        "trim",
        "trim_start",
        "trim_end",
        "chars",
        "char_indices",
        "split_at_checked",
    ];
    const PRODUCING: [&str; 2] = ["to_string", "push_str"];

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = TextSites::default();
        sites.visit_file(file.syntax());
        sites.outside_doors(file.path())
    }

    fn outside_doors(self, path: &Path) -> Vec<Violation> {
        if self.doors.is_empty() {
            self.found
                .into_iter()
                .map(|ident| {
                    Violation::new(
                        Site::new(path, LineNumber::from(ident.span())),
                        Finding::TextOutsideDoor { handler: Name::from(&ident) },
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn door_trait(item: &ItemImpl) -> Option<Ident> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| segment.ident.clone())
            .filter(|ident| Name::from(ident).matches::<Whole>(&Self::DOOR_TRAITS))
    }

    fn text_constructor(call: &ExprCall) -> Option<Ident> {
        let segments = match &*call.func {
            Expr::Path(path) => path.path.segments.iter().collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        match segments.as_slice() {
            [first, second] if first.ident == "String" && second.ident == "from" => Some(second.ident.clone()),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for TextSites {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.doors.extend(Self::door_trait(item));
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let name = Name::from(&call.method);
        self.found.extend(
            (name.matches::<Whole>(&Self::INSPECTING) || name.matches::<Whole>(&Self::PRODUCING))
                .then(|| call.method.clone()),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.found.extend(expr.mac.path.get_ident().filter(|ident| *ident == "format").cloned());
        syn::visit::visit_expr_macro(self, expr);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.found.extend(Self::text_constructor(call));
        syn::visit::visit_expr_call(self, call);
    }
}
