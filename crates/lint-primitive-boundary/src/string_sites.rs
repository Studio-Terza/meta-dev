mod text_form;

use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, Field, ImplItemConst, ImplItemFn, ImplItemType, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStatic,
    ItemStruct, ItemTrait, ItemType, Type, TypePath, TypeReference, Visibility,
};

use super::finding::Finding;
use lint_core::Gate;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;
pub use text_form::TextForm;

pub struct StringSites {
    path: PathBuf,
    context: Name,
    found: Vec<Violation>,
}

impl StringSites {
    pub fn exposed_in(file: &SourceFile) -> Vec<Violation> {
        let mut sites = StringSites { path: file.path().to_owned(), context: Name::default(), found: Vec::new() };
        sites.visit_file(file.syntax());
        sites.found
    }

    fn sighted(&mut self, form: TextForm, line: LineNumber) {
        let site = Site::new(&self.path, line);
        self.found.push(Violation::new(site, Finding::BareText { form, context: self.context.clone() }));
    }

    fn is_named(path: &TypePath, word: &str) -> bool {
        path.qself.is_none() && path.path.is_ident(word)
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

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.context = Name::from(&item.ident);
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

    fn visit_type_reference(&mut self, reference: &'ast TypeReference) {
        match &*reference.elem {
            Type::Path(path) if Self::is_named(path, "str") => {
                self.sighted(TextForm::Borrowed, LineNumber::from(reference.span()));
            }
            _ => syn::visit::visit_type_reference(self, reference),
        }
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        if Self::is_named(path, "String") {
            self.sighted(TextForm::Owned, LineNumber::from(path.span()));
        } else {
            syn::visit::visit_type_path(self, path);
        }
    }
}
