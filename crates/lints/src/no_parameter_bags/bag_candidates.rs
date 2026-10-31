mod candidate;
mod method_shape;

pub use candidate::Candidate;

use std::collections::{BTreeMap, BTreeSet};

use method_shape::MethodShape;
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl, ItemMod, ItemStruct, Type};

use lint_core::LineNumber;
use lint_core::Name;
use lint_core::SourceFile;
use lint_core::Walk;

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

    pub fn candidates(&self) -> impl Iterator<Item = Candidate> {
        self.structs
            .iter()
            .filter(|(name, _)| !self.disqualified.contains(*name))
            .map(|(name, line)| Candidate::new(name.clone(), *line))
    }

    fn self_name(item: &ItemImpl) -> Option<Name> {
        match &*item.self_ty {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    fn carries_logic(item: &ItemImpl, self_name: &Name) -> bool {
        item.trait_.is_some()
            || item.items.iter().any(|member| match member {
                ImplItem::Fn(function) => MethodShape::classify(function, self_name) == MethodShape::Logic,
                _ => false,
            })
    }
}

impl<'ast> Visit<'ast> for BagCandidates {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.structs.insert(Name::from(&item.ident), LineNumber::from(item.ident.span()));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.disqualified.extend(Self::self_name(item).filter(|name| Self::carries_logic(item, name)));
    }
}
