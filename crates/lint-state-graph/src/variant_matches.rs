mod variant_paths;

use variant_paths::VariantPaths;

use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{Arm, Expr, ExprLet, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Local, Pat};

use super::declarations::Declarations;
use super::finding::Finding;
use lint_core::Home;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct VariantMatches<'scan, 'file> {
    declarations: &'scan Declarations,
    path: &'file Path,
    home: Home,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> VariantMatches<'scan, 'file> {
    pub fn violations(file: &'file SourceFile, declarations: &'scan Declarations) -> Vec<Violation<'file>> {
        let mut matches =
            VariantMatches { declarations, path: file.path(), home: Home::containing(file), violations: Vec::new() };
        matches.visit_file(file.syntax());
        matches.violations
    }

    fn foreign_variant(&self, pat: &Pat) -> Option<Name> {
        VariantPaths::in_pat(pat)
            .into_iter()
            .find_map(|enum_name| self.declarations.foreign_sum_type(&enum_name, &self.home).map(|_| enum_name))
    }

    fn violation(&self, pat: &Pat, enum_name: &Name) -> Violation<'file> {
        Violation::new(Site::new(self.path, LineNumber::from(pat.span())), Finding::ForeignMatch { enum_name })
    }

    fn foreign_match(&self, arm: &Arm) -> Option<Violation<'file>> {
        match Self::table(&arm.body) {
            Some(_) => None,
            None => self.foreign_variant(&arm.pat).map(|enum_name| self.violation(&arm.pat, &enum_name)),
        }
    }

    fn table(expr: &Expr) -> Option<&Expr> {
        match expr {
            Expr::Lit(_) | Expr::Path(_) => Some(expr),
            Expr::Unary(unary) => Self::table(&unary.expr).map(|_| expr),
            Expr::Paren(paren) => Self::table(&paren.expr).map(|_| expr),
            Expr::Tuple(tuple) => tuple.elems.iter().map(Self::table).collect::<Option<Vec<_>>>().map(|_| expr),
            Expr::Array(array) => array.elems.iter().map(Self::table).collect::<Option<Vec<_>>>().map(|_| expr),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for VariantMatches<'_, '_> {
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
