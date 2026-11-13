mod candidate;
mod constructor;

use std::path::Path;

use syn::{Ident, ImplItem, ImplItemFn, ItemEnum, ItemImpl, ItemMod, Type, Visibility};

use constructor::Constructor;
use lint_core::{Site, Violation};
use lint_core::{SourceFile, Walk};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Descend, Finds, Kind, Watch};

use std::collections::BTreeSet;

use crate::finding::Finding;
use candidate::Candidate;

#[derive(Debug)]
pub(crate) struct Offers<'file, 'run> {
    path: &'file Path,
    names: &'run BTreeSet<Name>,
    found: Vec<Candidate<'file>>,
}

impl<'file, 'run> Offers<'file, 'run> {
    pub(crate) fn new(file: &'file SourceFile, names: &'run BTreeSet<Name>) -> Offers<'file, 'run> {
        Offers { path: file.head().path(), names, found: Vec::new() }
    }

    fn offered(&mut self, ident: &Ident, visibility: &Visibility) {
        let violation = Violation::new(Site::new(self.path, LineNumber::from(ident.span())), Finding::OnlyTests);
        self.found.extend(
            matches!(visibility, Visibility::Public(_) | Visibility::Restricted(_))
                .then_some(Candidate::new(Name::from(ident), violation)),
        );
    }

    fn constructors(&mut self, item: &ItemImpl, owner: &Ident) {
        item.items
            .iter()
            .filter_map(Self::function)
            .filter(|function| Walk::outside_tests(&function.attrs) == Walk::Entered)
            .filter_map(|function| Constructor::made_by(&function.sig, owner).map(|_| function))
            .for_each(|function| self.offered(&function.sig.ident, &function.vis));
    }

    fn function(member: &ImplItem) -> Option<&ImplItemFn> {
        match member {
            ImplItem::Fn(function) => Some(function),
            _ => None,
        }
    }

    fn owner(item: &ItemImpl) -> Option<&Ident> {
        match &*item.self_ty {
            Type::Path(path) => path.path.segments.last().map(|last| &last.ident),
            _ => None,
        }
    }
}

impl<'ast> Watch<'ast> for Offers<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemEnum]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        let owner = Self::owner(item).filter(|_| Walk::inherent_impls(item) == Walk::Entered);
        let entered = Descend::from(Walk::outside_tests(&item.attrs)).entered();
        entered.and(owner).into_iter().for_each(|named| self.constructors(item, named));
        Descend::Over
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        let entered = Descend::from(Walk::outside_tests(&item.attrs)).entered();
        entered.into_iter().flat_map(|_| &item.variants).for_each(|variant| self.offered(&variant.ident, &item.vis));
        Descend::Over
    }
}

impl<'file> Finds<'file> for Offers<'file, '_> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let names = self.names;
        self.found
            .into_iter()
            .filter(|candidate| names.contains(candidate.called()))
            .map(Candidate::violation)
            .collect()
    }
}
