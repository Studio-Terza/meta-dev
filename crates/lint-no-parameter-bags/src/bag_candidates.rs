mod candidate;
mod method_shape;

pub use candidate::Candidate;

use std::collections::{BTreeMap, BTreeSet};

use method_shape::MethodShape;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, ItemStruct};

use lint_core::{SourceFile, Violation};
use lint_sweep::{Descend, Finds, Kind, Watch};

use crate::Bags;
use lint_core::Syntax;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Debug)]
pub struct BagCandidates<'file, 'run> {
    file: &'file SourceFile,
    bags: &'run Bags<'run>,
    structs: BTreeMap<Name, LineNumber>,
    disqualified: BTreeSet<Name>,
}

impl<'file, 'run> BagCandidates<'file, 'run> {
    pub fn new(file: &'file SourceFile, bags: &'run Bags<'run>) -> BagCandidates<'file, 'run> {
        BagCandidates { file, bags, structs: BTreeMap::new(), disqualified: BTreeSet::new() }
    }

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

impl<'ast> Watch<'ast> for BagCandidates<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ItemImpl]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.structs.insert(Name::from(&item.ident), LineNumber::from(item.ident.span()));
        Descend::Over
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        self.disqualified.extend(Name::of_type(&item.self_ty).and_then(|name| Self::logic_carrier(item, name)));
        Descend::Over
    }
}

impl<'file> Finds<'file> for BagCandidates<'file, '_> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let file = self.file;
        let bags = self.bags;
        self.candidates()
            .filter(|candidate| bags.exports.exported(file, candidate.name()).is_none())
            .filter(|candidate| bags.one_consumer(file, candidate.name()).is_some())
            .map(|candidate| candidate.violation(file.head().path()))
            .collect()
    }
}
