mod candidate;
mod method_shape;

pub use candidate::Candidate;

use std::collections::{BTreeMap, BTreeSet};

use method_shape::MethodShape;
use syn::visit::Visit;
use syn::{ImplItem, ImplItemFn, ItemImpl, ItemMod, ItemStruct};

use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Default)]
pub struct BagCandidates {
    structs: BTreeMap<Name, LineNumber>,
    disqualified: BTreeSet<Name>,
}

impl BagCandidates {
    pub fn in_file(file: &SourceFile) -> BagCandidates {
        let mut candidates = BagCandidates::default();
        candidates.visit_file(file.syntax());
        candidates
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

impl<'ast> Visit<'ast> for BagCandidates {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.structs.insert(Name::from(&item.ident), LineNumber::from(item.ident.span()));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.disqualified.extend(Name::of_type(&item.self_ty).and_then(|name| Self::logic_carrier(item, name)));
    }
}
