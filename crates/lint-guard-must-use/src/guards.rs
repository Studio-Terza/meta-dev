use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use syn::visit::{self, Visit};
use syn::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, Type};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name};

use super::Finding;

#[derive(Debug, Default)]
pub struct Guards {
    path: PathBuf,
    dropped: BTreeSet<Name>,
    unmarked: BTreeMap<Site, Name>,
}

impl Guards {
    const DROP: &str = "Drop";
    const MUST_USE: &str = "must_use";

    pub fn scan(&mut self, file: &SourceFile) {
        file.path().clone_into(&mut self.path);
        self.visit_file(file.syntax());
    }

    pub fn violations(self) -> Vec<Violation> {
        let Guards { dropped, unmarked, .. } = self;
        unmarked
            .into_iter()
            .filter(|(_, name)| dropped.contains(name))
            .map(|(site, name)| Violation::new(site, Finding::from(&name)))
            .collect()
    }

    fn dropped_type(item: &ItemImpl) -> Option<Name> {
        item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).filter(|last| last.ident == Self::DROP)?;
        match item.self_ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident) {
        let site = Site::new(&self.path, LineNumber::from(name.span()));
        self.unmarked.extend(
            (!attributes.iter().any(|attribute| attribute.path().is_ident(Self::MUST_USE)))
                .then(|| (site, Name::from(name))),
        );
    }
}

impl<'ast> Visit<'ast> for Guards {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.dropped.extend(Self::dropped_type(item)));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.declared(&item.attrs, &item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.declared(&item.attrs, &item.ident));
    }
}
