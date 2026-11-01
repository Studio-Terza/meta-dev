mod text_form;

use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, Field, ForeignItemFn, ImplItemConst, ImplItemFn, ImplItemType, ItemConst, ItemEnum, ItemFn, ItemImpl,
    ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, TypePath, Visibility,
};

use super::finding::Finding;
use super::number_sites::UseAliases;
use lint_core::Gate;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::{Word, words};
pub use text_form::TextForm;

pub struct StringSites {
    path: PathBuf,
    aliases: UseAliases,
    context: Name,
    found: Vec<Violation>,
}

impl StringSites {
    const OWNED: [&'static [Word<'static>]; 3] =
        [&words!["String"], &words!["std", "string", "String"], &words!["alloc", "string", "String"]];
    const UNSIZED: [Word<'static>; 1] = words!["str"];

    pub fn exposed_in(file: &SourceFile) -> Vec<Violation> {
        let mut sites = StringSites {
            path: file.path().to_owned(),
            aliases: UseAliases::in_file(file.syntax()),
            context: Name::default(),
            found: Vec::new(),
        };
        sites.visit_file(file.syntax());
        sites.found
    }

    fn sighted(&mut self, form: TextForm, line: LineNumber) {
        let site = Site::new(&self.path, line);
        self.found.push(Violation::new(site, Finding::BareText { form, context: self.context.clone() }));
    }

    fn form_of(&self, path: &TypePath) -> Option<TextForm> {
        Some(path).filter(|path| path.qself.is_none()).map(|path| self.aliases.resolve(&path.path)).and_then(
            |spelled| {
                Self::OWNED
                    .iter()
                    .find(|owned| spelled.spells(owned))
                    .map(|_| TextForm::Owned)
                    .or_else(|| spelled.spells(&Self::UNSIZED).then_some(TextForm::Unsized))
            },
        )
    }

    fn exposed(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public(_))
    }
}

impl<'ast> Visit<'ast> for StringSites {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered if !Gate::PORTS.names(module) => syn::visit::visit_item_mod(self, module),
            Walk::Entered | Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_trait(&mut self, _: &'ast ItemTrait) {}

    fn visit_block(&mut self, _: &'ast Block) {}

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_item_static(&mut self, _: &'ast ItemStatic) {}

    fn visit_item_type(&mut self, _: &'ast ItemType) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_impl_item_type(&mut self, _: &'ast ImplItemType) {}

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.context = Name::from(&item.sig.ident);
        Self::exposed(&item.vis).then_some(&item.sig).iter().for_each(|sig| self.visit_signature(sig));
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.context = Name::from(&item.sig.ident);
        Self::exposed(&item.vis).then_some(&item.sig).iter().for_each(|sig| self.visit_signature(sig));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.context = Name::from(&item.ident);
        item.fields.named.iter().filter(|field| Self::exposed(&field.vis)).for_each(|field| self.visit_field(field));
    }

    fn visit_foreign_item_fn(&mut self, item: &'ast ForeignItemFn) {
        self.context = Name::from(&item.sig.ident);
        Self::exposed(&item.vis).then_some(&item.sig).iter().for_each(|sig| self.visit_signature(sig));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.context = Name::from(&item.ident);
        Self::exposed(&item.vis).then_some(&item.generics).iter().for_each(|generics| self.visit_generics(generics));
        item.fields.iter().filter(|field| Self::exposed(&field.vis)).for_each(|field| self.visit_field(field));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.context = Name::from(&item.ident);
        Some(item)
            .filter(|item| Self::exposed(&item.vis))
            .iter()
            .for_each(|item| syn::visit::visit_item_enum(self, item));
    }

    fn visit_field(&mut self, field: &'ast Field) {
        self.context = field.ident.as_ref().map_or_else(|| self.context.clone(), Name::from);
        syn::visit::visit_type(self, &field.ty);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        match self.form_of(path) {
            Some(form) => self.sighted(form, LineNumber::from(path.span())),
            None => syn::visit::visit_type_path(self, path),
        }
    }
}
