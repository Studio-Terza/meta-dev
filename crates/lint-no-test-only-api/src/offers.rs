mod constructor;

use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Ident, ImplItem, ImplItemFn, ItemEnum, ItemImpl, ItemMod, Type, Visibility};

use constructor::Constructor;
use lint_core::{Site, Violation};
use lint_core::{SourceFile, Walk};
use lint_layout::{LineNumber, Name};

use crate::candidate::Candidate;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Offers<'file> {
    path: &'file Path,
    found: Vec<Candidate>,
}

impl<'file> Offers<'file> {
    pub(crate) fn in_file(file: &'file SourceFile) -> Vec<Candidate> {
        let mut offers = Offers { path: file.path(), found: Vec::new() };
        Some(file).filter(|file| file.is_shipped()).into_iter().for_each(|file| offers.visit_file(file.syntax()));
        offers.found
    }

    fn offered(&mut self, ident: &Ident, visibility: &Visibility) {
        let violation = Violation::new(Site::new(self.path, LineNumber::from(ident.span())), Finding::OnlyTests);
        self.found.extend(
            (!matches!(visibility, Visibility::Inherited)).then_some(Candidate::new(Name::from(ident), violation)),
        );
    }

    fn constructors(&mut self, item: &ItemImpl, owner: &Ident) {
        item.items
            .iter()
            .filter_map(Self::function)
            .filter(|function| Walk::outside_tests(&function.attrs) == Walk::Entered)
            .filter(|function| Constructor::is_made_by(&function.sig, owner))
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

impl<'ast> Visit<'ast> for Offers<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let owner = Self::owner(item).filter(|_| Walk::inherent_impls(item) == Walk::Entered);
        Walk::enter(&item.attrs, || owner.iter().for_each(|owner| self.constructors(item, owner)));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || {
            item.variants.iter().for_each(|variant| self.offered(&variant.ident, &item.vis));
        });
    }
}
