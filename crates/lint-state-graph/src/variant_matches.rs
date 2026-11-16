mod variant_paths;

use variant_paths::VariantPaths;

use std::path::Path;
use syntax::Spanned;

use syntax::{Arm, Constant, ExprLet, ExprMatch, ImplItemFn, ItemFn, ItemImpl, ItemMod, Local, Pat};

use super::declarations::Declarations;
use super::finding::Finding;
use lint_core::Home;
use lint_core::Site;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Kind, Watch};
use lint_sweep::{Gathers, Watching};

#[derive(Debug)]
pub struct VariantMatches<'file> {
    path: &'file Path,
    home: Home<'file>,
    candidates: Vec<Matched<'file>>,
}

#[derive(Debug)]
struct Matched<'file> {
    site: Site<'file>,
    names: Vec<Name>,
}

#[derive(Debug)]
pub struct MatchCandidates<'file> {
    home: Home<'file>,
    candidates: Vec<Matched<'file>>,
}

impl<'file> MatchCandidates<'file> {
    pub fn foreign(self, declarations: &Declarations<'file>) -> Vec<Violation<'file>> {
        let home = self.home;
        self.candidates
            .into_iter()
            .filter_map(|matched| {
                let foreign =
                    matched.names.iter().find(|enum_name| declarations.foreign_sum_type(enum_name, &home).is_some())?;
                Some(Violation::new(matched.site, Finding::ForeignMatch { enum_name: foreign }))
            })
            .collect()
    }
}

impl<'file> VariantMatches<'file> {
    pub fn new(path: &'file Path, home: Home<'file>) -> Self {
        VariantMatches { path, home, candidates: Vec::new() }
    }

    fn matched(&self, pat: &Pat) -> Matched<'file> {
        Matched { site: Site::new(self.path, LineNumber::from(pat.span())), names: VariantPaths::in_pat(pat) }
    }

    fn foreign_match(&self, arm: &Arm) -> Option<Matched<'file>> {
        match Constant::composed(&arm.body) {
            Some(_) => None,
            None => Some(self.matched(&arm.pat)),
        }
    }
}

impl<'file> Gathers<MatchCandidates<'file>> for VariantMatches<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        vec![self]
    }

    fn gathered(self) -> MatchCandidates<'file> {
        MatchCandidates { home: self.home, candidates: self.candidates }
    }
}

impl<'ast> Watch<'ast> for VariantMatches<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemFn, Kind::ImplItemFn, Kind::ExprMatch, Kind::ExprLet, Kind::Local]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_impl(&mut self, _: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_fn(&mut self, _: &'ast ItemFn, facts: &Facts<'_, ItemFn>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_impl_item_fn(&mut self, _: &'ast ImplItemFn, facts: &Facts<'_, ImplItemFn>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_expr_match(&mut self, expr: &'ast ExprMatch, _: &Facts<'_, ExprMatch>) -> Descend {
        let found: Vec<Matched<'_>> = expr.arms.iter().filter_map(|arm| self.foreign_match(arm)).collect();
        self.candidates.extend(found);
        Descend::Into
    }

    fn on_expr_let(&mut self, expr: &'ast ExprLet, _: &Facts<'_, ExprLet>) -> Descend {
        let found = self.matched(&expr.pat);
        self.candidates.push(found);
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local, _: &Facts<'_, Local>) -> Descend {
        let found = local.init.as_ref().filter(|init| init.diverge.is_some()).map(|_| self.matched(&local.pat));
        self.candidates.extend(found);
        Descend::Into
    }
}
