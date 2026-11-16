mod candidate;
mod method_shape;

pub use candidate::Candidate;

use std::collections::{BTreeMap, BTreeSet};

use method_shape::MethodShape;
use syntax::{ImplItem, ImplItemFn, ItemImpl, ItemMod, ItemStruct};

use lint_core::{SourceFile, Violation};
use lint_sweep::{Descend, Facts, Gathers, Kind, Watch, Watching};

use crate::Bags;
use lint_core::Syntax;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Debug)]
pub struct BagCandidates<'file> {
    file: &'file SourceFile,
    structs: BTreeMap<Name, LineNumber>,
    disqualified: BTreeSet<Name>,
}

impl BagCandidates<'_> {
    pub fn candidates(&self) -> impl Iterator<Item = Candidate<'_>> {
        self.structs
            .iter()
            .filter(|(name, _)| self.disqualified.iter().all(|disqualified| disqualified != *name))
            .map(|(name, line)| Candidate::new(name, *line))
    }

    fn logic_carrier(item: &ItemImpl, self_name: Name) -> Option<Name> {
        match &item.trait_ {
            Some(_) => Some(self_name),
            None => item.items.iter().find_map(|member| Self::logic(member, &self_name)).map(|_| self_name),
        }
    }

    fn logic<'member>(member: &'member ImplItem, self_name: &Name) -> Option<&'member ImplItemFn> {
        Syntax::impl_fn(member).filter(|function| MethodShape::classify(function, self_name) == MethodShape::Logic)
    }
}

impl<'ast> Watch<'ast> for BagCandidates<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ItemImpl]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct, facts: &Facts<'_, ItemStruct>) -> Descend {
        self.structs.extend(facts.name().map(|name| (name.clone(), LineNumber::from(item.ident.span()))));
        Descend::Over
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        self.disqualified.extend(facts.name().cloned().and_then(|name| Self::logic_carrier(item, name)));
        Descend::Over
    }
}

impl<'file> Gathers<BagCandidates<'file>> for BagCandidates<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        vec![self]
    }

    fn gathered(self) -> BagCandidates<'file> {
        self
    }
}

impl<'file> BagCandidates<'file> {
    pub fn bagged(self, bags: &Bags<'_>) -> Vec<Violation<'file>> {
        let file = self.file;
        self.candidates()
            .filter(|candidate| bags.exports.exported(file, candidate.name()).is_none())
            .filter(|candidate| bags.one_consumer(file, candidate.name()).is_some())
            .map(|candidate| candidate.violation(file.head().path()))
            .collect()
    }
}

impl<'file> From<&'file SourceFile> for BagCandidates<'file> {
    fn from(file: &'file SourceFile) -> BagCandidates<'file> {
        BagCandidates { file, structs: BTreeMap::new(), disqualified: BTreeSet::new() }
    }
}
