mod variant_paths;

use variant_paths::VariantPaths;

use std::path::Path;
use syn::spanned::Spanned;

use syn::{Arm, Expr, ExprLet, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Local, Pat};

use super::declarations::Declarations;
use super::finding::Finding;
use lint_core::Home;
use lint_core::Site;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Kind, Watch};

#[derive(Debug)]
pub struct VariantMatches<'scan, 'file> {
    declarations: &'scan Declarations<'file>,
    path: &'file Path,
    home: Home<'file>,
    violations: Vec<Violation<'file>>,
}

impl<'scan, 'file> VariantMatches<'scan, 'file> {
    pub fn new(path: &'file Path, home: Home<'file>, declarations: &'scan Declarations<'file>) -> Self {
        VariantMatches { declarations, path, home, violations: Vec::new() }
    }

    pub fn violations(self) -> Vec<Violation<'file>> {
        self.violations
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

impl<'ast> Watch<'ast> for VariantMatches<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemFn, Kind::ImplItemFn, Kind::ExprMatch, Kind::ExprLet, Kind::Local]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_expr_match(&mut self, expr: &'ast ExprMatch) -> Descend {
        let found: Vec<Violation> = expr.arms.iter().filter_map(|arm| self.foreign_match(arm)).collect();
        self.violations.extend(found);
        Descend::Into
    }

    fn on_expr_let(&mut self, expr: &'ast ExprLet) -> Descend {
        let found = self.foreign_variant(&expr.pat).map(|enum_name| self.violation(&expr.pat, &enum_name));
        self.violations.extend(found);
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        let found = local
            .init
            .as_ref()
            .filter(|init| init.diverge.is_some())
            .and_then(|_| self.foreign_variant(&local.pat))
            .map(|enum_name| self.violation(&local.pat, &enum_name));
        self.violations.extend(found);
        Descend::Into
    }
}
