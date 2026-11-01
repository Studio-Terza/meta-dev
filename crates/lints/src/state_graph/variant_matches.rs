mod variant_paths;

use variant_paths::VariantPaths;

use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{Arm, Expr, ExprLet, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Local, Pat};

use super::declarations::Declarations;
use super::finding::Finding;
use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

pub struct VariantMatches<'scan> {
    declarations: &'scan Declarations,
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl<'scan> VariantMatches<'scan> {
    pub fn violations(file: &'scan SourceFile, declarations: &'scan Declarations) -> Vec<Violation> {
        let mut matches = VariantMatches { declarations, path: file.path(), violations: Vec::new() };
        matches.visit_file(file.syntax());
        matches.violations
    }

    fn foreign_variant(&self, pat: &Pat) -> Option<Name> {
        VariantPaths::in_pat(pat)
            .into_iter()
            .find(|enum_name| self.declarations.is_foreign_sum_type(enum_name, self.path))
    }

    fn violation(&self, pat: &Pat, enum_name: &Name) -> Violation {
        Violation::new(
            Site::new(self.path, LineNumber::from(pat.span())),
            Finding::ForeignMatch { enum_name: enum_name.clone() },
        )
    }

    fn foreign_match(&self, arm: &Arm) -> Option<Violation> {
        self.foreign_variant(&arm.pat)
            .filter(|_| !Self::is_table(&arm.body))
            .map(|enum_name| self.violation(&arm.pat, &enum_name))
    }

    fn is_table(expr: &Expr) -> bool {
        match expr {
            Expr::Lit(_) | Expr::Path(_) => true,
            Expr::Unary(unary) => Self::is_table(&unary.expr),
            Expr::Paren(paren) => Self::is_table(&paren.expr),
            Expr::Tuple(tuple) => tuple.elems.iter().all(Self::is_table),
            Expr::Array(array) => array.elems.iter().all(Self::is_table),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for VariantMatches<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || syn::visit::visit_item_impl(self, item));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_impl_item_fn(self, function));
    }

    fn visit_expr_match(&mut self, expr: &'ast ExprMatch) {
        let found: Vec<Violation> = expr.arms.iter().filter_map(|arm| self.foreign_match(arm)).collect();
        self.violations.extend(found);
        syn::visit::visit_expr_match(self, expr);
    }

    fn visit_expr_let(&mut self, expr: &'ast ExprLet) {
        let found = self.foreign_variant(&expr.pat).map(|enum_name| self.violation(&expr.pat, &enum_name));
        self.violations.extend(found);
        syn::visit::visit_expr_let(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let found = local
            .init
            .as_ref()
            .filter(|init| init.diverge.is_some())
            .and_then(|_| self.foreign_variant(&local.pat))
            .map(|enum_name| self.violation(&local.pat, &enum_name));
        self.violations.extend(found);
        syn::visit::visit_local(self, local);
    }
}
