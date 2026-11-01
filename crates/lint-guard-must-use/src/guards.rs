mod type_key;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Meta, Token, Type};

use lint_core::{Site, SourceFile, Violation, Walk};
use lint_layout::{LineNumber, Name};
use lint_paths::Module;

use super::Finding;
use type_key::TypeKey;

#[derive(Debug, Default)]
pub struct Guards {
    path: PathBuf,
    module: Module,
    declared: BTreeSet<TypeKey>,
    unmarked: BTreeMap<Site, TypeKey>,
    aliases: BTreeMap<TypeKey, Name>,
    dropped: Vec<TypeKey>,
}

impl Guards {
    const DROP: &str = "Drop";
    const MUST_USE: &str = "must_use";
    const CONDITIONAL: &str = "cfg_attr";

    pub fn scan(&mut self, file: &SourceFile) {
        file.path().clone_into(&mut self.path);
        self.module = Module::of_file(file);
        self.visit_file(file.syntax());
    }

    pub fn violations(self) -> Vec<Violation> {
        let bound: BTreeSet<TypeKey> = self.dropped.iter().flat_map(|key| self.bound(key)).collect();
        self.unmarked
            .into_iter()
            .filter(|(_, key)| bound.contains(key))
            .map(|(site, key)| Violation::new(site, Finding::from(key.type_name())))
            .collect()
    }

    fn bound(&self, written: &TypeKey) -> Vec<TypeKey> {
        let key = written.aliased(&self.aliases);
        if self.declared.contains(&key) {
            vec![key]
        } else {
            self.declared
                .iter()
                .filter(|other| other.type_name() == key.type_name() && other.shares_crate(&key))
                .cloned()
                .collect()
        }
    }

    fn dropped_type(&self, item: &ItemImpl) -> Option<TypeKey> {
        item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).filter(|last| last.ident == Self::DROP)?;
        match item.self_ty.as_ref() {
            Type::Path(path) => {
                path.path.segments.last().map(|segment| TypeKey::new(self.module.clone(), Name::from(&segment.ident)))
            }
            _ => None,
        }
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident) {
        let site = Site::new(&self.path, LineNumber::from(name.span()));
        let key = TypeKey::new(self.module.clone(), Name::from(name));
        self.declared.insert(key.clone());
        self.unmarked.extend((!attributes.iter().any(Self::marks)).then_some((site, key)));
    }

    fn marks(attribute: &Attribute) -> bool {
        attribute.path().is_ident(Self::MUST_USE)
            || (attribute.path().is_ident(Self::CONDITIONAL)
                && attribute
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .is_ok_and(|metas| metas.iter().skip(1).any(|meta| meta.path().is_ident(Self::MUST_USE))))
    }
}

impl<'ast> Visit<'ast> for Guards {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.module.clone();
        self.module = outer.within(&Name::from(&module.ident));
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        self.module = outer;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.dropped.extend(self.dropped_type(item)));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        Walk::enter(&item.attrs, || self.declared(&item.attrs, &item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        Walk::enter(&item.attrs, || self.declared(&item.attrs, &item.ident));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        Walk::enter(&item.attrs, || self.declared(&item.attrs, &item.ident));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        let target = match item.ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        };
        let key = TypeKey::new(self.module.clone(), Name::from(&item.ident));
        Walk::enter(&item.attrs, || self.aliases.extend(target.map(|target| (key, target))));
    }
}
