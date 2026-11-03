mod owner;
pub use owner::Owner;
mod sighting;

use std::path::Path;

use syn::visit::Visit;
use syn::{ExprCall, ExprMacro, ExprMethodCall, Ident, ItemImpl, ItemMod};

use lint_core::Gate;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_file::FileLint;
use lint_layout::{Name, Whole, Word, words};

use sighting::Sighting;

#[derive(Default)]
pub struct TextSites {
    doors: Vec<Owner>,
    owner: Owner,
    found: Vec<Sighting>,
}

impl TextSites {
    const DOOR_TRAITS: [Word<'static>; 6] = words!["FromStr", "TryFrom", "Display", "AsRef", "Borrow", "Deref"];
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

    fn into_violations(self, path: &Path) -> Vec<Violation<'_>> {
        self.found
            .into_iter()
            .filter_map(|sighting| sighting.outside(&self.doors))
            .map(|sighting| sighting.into_violation(path))
            .collect()
    }

    fn door_trait(item: &ItemImpl) -> Option<Ident> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| segment.ident.clone())
            .filter(|ident| Name::from(ident).matches::<Whole>(&Self::DOOR_TRAITS).is_some())
    }

    fn sighted(&mut self, ident: Option<Ident>) {
        let owner = self.owner.clone();
        self.found.extend(ident.map(|ident| Sighting::new(owner, ident)));
    }

    fn text_constructor(call: &ExprCall) -> Option<Ident> {
        Syntax::expr_path(&call.func)
            .map(|path| &path.path.segments)
            .filter(|segments| segments.len() == 2)
            .filter(|segments| segments.first().is_some_and(|first| first.ident == "String"))
            .and_then(|segments| segments.last())
            .filter(|second| second.ident == "from")
            .map(|second| second.ident.clone())
    }
}

impl FileLint for TextSites {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut sites = TextSites::default();
        sites.visit_file(file.syntax());
        sites.into_violations(file.path())
    }
}

impl<'ast> Visit<'ast> for TextSites {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::BOUNDARY.enter_outside(module, || syn::visit::visit_item_mod(self, module));
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
            name.matches::<Whole>(&Self::INSPECTING)
                .or_else(|| name.matches::<Whole>(&Self::PRODUCING))
                .map(|_| call.method.clone()),
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
