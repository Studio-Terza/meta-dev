mod owner;
mod sighting;

use std::path::Path;

use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall, Ident, ItemImpl, ItemMod};

use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::{Name, Whole, Word, words};
use owner::Owner;
use sighting::Sighting;

#[derive(Default)]
pub struct TextSites {
    doors: Vec<Owner>,
    owner: Owner,
    found: Vec<Sighting>,
}

impl TextSites {
    const DOOR_TRAITS: [Word<'static>; 3] = words!["FromStr", "TryFrom", "Display"];
    const INSPECTING: [Word<'static>; 8] = words![
        "split_whitespace",
        "split_once",
        "trim",
        "trim_start",
        "trim_end",
        "chars",
        "char_indices",
        "split_at_checked",
    ];
    const PRODUCING: [Word<'static>; 2] = words!["to_string", "push_str"];

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = TextSites::default();
        sites.visit_file(file.syntax());
        sites.into_violations(file.path())
    }

    fn into_violations(self, path: &Path) -> Vec<Violation> {
        self.found
            .into_iter()
            .filter(|sighting| sighting.outside(&self.doors))
            .map(|sighting| sighting.into_violation(path))
            .collect()
    }

    fn door_trait(item: &ItemImpl) -> Option<Ident> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| segment.ident.clone())
            .filter(|ident| Name::from(ident).matches::<Whole>(&Self::DOOR_TRAITS))
    }

    fn sighted(&mut self, ident: Option<Ident>) {
        let owner = self.owner.clone();
        self.found.extend(ident.map(|ident| Sighting::new(owner, ident)));
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
        let outer = std::mem::replace(&mut self.owner, Owner::of_impl(item));
        self.doors.extend(Self::door_trait(item).map(|_| self.owner.clone()));
        syn::visit::visit_item_impl(self, item);
        self.owner = outer;
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let name = Name::from(&call.method);
        self.sighted(
            (name.matches::<Whole>(&Self::INSPECTING) || name.matches::<Whole>(&Self::PRODUCING))
                .then(|| call.method.clone()),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.sighted(expr.mac.path.get_ident().filter(|ident| *ident == "format").cloned());
        syn::visit::visit_expr_macro(self, expr);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.sighted(Self::text_constructor(call));
        syn::visit::visit_expr_call(self, call);
    }
}
