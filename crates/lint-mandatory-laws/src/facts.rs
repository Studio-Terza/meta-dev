mod obligation;
mod profile;
mod shape;

use std::collections::BTreeMap;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{ItemEnum, ItemImpl, ItemStruct};

use lint_layout::Name;
pub(crate) use obligation::Obligation;
use profile::Profile;
use shape::Shape;

#[derive(Debug, Default)]
pub(crate) struct Facts {
    profiles: Vec<Profile>,
}

impl Facts {
    pub(crate) fn collect_from(file: &syn::File) -> Facts {
        let mut facts = Facts::default();
        facts.visit_file(file);
        facts
    }

    pub(crate) fn obligations(self, path: &Path) -> Vec<Obligation> {
        let merged = self.profiles.into_iter().fold(BTreeMap::<Name, Profile>::new(), |mut merged, profile| {
            let owner = profile.owner().clone();
            let combined = match merged.remove(&owner) {
                Some(earlier) => earlier.merged(profile),
                None => profile,
            };
            merged.insert(owner, combined);
            merged
        });
        merged.values().flat_map(|profile| profile.obligations(path)).collect()
    }
}

impl<'ast> Visit<'ast> for Facts {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.profiles.push(Shape::of_struct(item));
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.profiles.push(Shape::of_enum(item));
        visit::visit_item_enum(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.profiles.extend(Shape::of_impl(item));
        visit::visit_item_impl(self, item);
    }
}
