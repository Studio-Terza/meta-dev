mod members;
mod modules;
mod type_key;

use std::mem;
use std::path::Path;

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Meta, MetaList, Token};

use lint_core::{Site, SourceFile, Syntax, Walk};
use lint_layout::{LineNumber, Name};
use lint_paths::Module;

use members::Members;
pub use modules::Modules;

#[derive(Debug)]
pub struct Guards<'file> {
    path: &'file Path,
    module: Module<'file>,
    members: Members<'file>,
    modules: Modules<'file>,
}

impl<'file> Guards<'file> {
    const DROP: &str = "Drop";

    pub fn scanned(files: &[&'file SourceFile]) -> Modules<'file> {
        let empty = Guards {
            path: Path::new(""),
            module: Module::default(),
            members: Members::default(),
            modules: Modules::default(),
        };
        files
            .iter()
            .fold(empty, |guards, file| {
                let mut next = Guards { path: file.path(), module: Module::of_file(file), ..guards };
                next.visit_file(file.syntax());
                next.modules.file(mem::take(&mut next.module), mem::take(&mut next.members));
                next
            })
            .modules
    }

    fn dropped_type(item: &ItemImpl) -> Option<Name> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|last| last.ident == Self::DROP)
            .and_then(|_| Syntax::last_name(&item.self_ty))
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident) {
        let site = Site::new(self.path, LineNumber::from(name.span()));
        self.members.declare(Name::from(name), attributes.iter().find_map(MustUse::mark).map_or(Some(site), |_| None));
    }
}

struct MustUse;

impl MustUse {
    const MUST_USE: &str = "must_use";
    const CONDITIONAL: &str = "cfg_attr";

    fn mark(attribute: &Attribute) -> Option<&Attribute> {
        Some(attribute).filter(|attribute| attribute.path().is_ident(Self::MUST_USE)).or_else(|| {
            attribute
                .meta
                .require_list()
                .ok()
                .filter(|list| list.path.is_ident(Self::CONDITIONAL))
                .and_then(Self::conditional)
                .map(|_| attribute)
        })
    }

    fn conditional(list: &MetaList) -> Option<Meta> {
        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()
            .and_then(|nested| nested.into_iter().skip(1).find(|meta| meta.path().is_ident(Self::MUST_USE)))
    }
}

impl<'ast> Visit<'ast> for Guards<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let inner = self.module.within(Name::from(&module.ident));
        let outer = mem::replace(&mut self.module, inner);
        let around = mem::take(&mut self.members);
        Walk::enter(&module.attrs, || visit::visit_item_mod(self, module));
        let inner = mem::replace(&mut self.module, outer);
        let within = mem::replace(&mut self.members, around);
        self.modules.file(inner, within);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || self.members.drop_implemented(Self::dropped_type(item)));
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
        let target = Syntax::last_name(&item.ty);
        Walk::enter(&item.attrs, || self.members.alias(Name::from(&item.ident), target));
    }
}
