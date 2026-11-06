mod scan;

use std::path::Path;

use scan::Scan;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Field, ForeignItemStatic, Ident, ImplItemConst, ImplItemFn, ImplItemType, ItemConst, ItemImpl, ItemMod, ItemStatic,
    ItemStruct, ItemType, Macro, Signature, TraitItemConst, TraitItemType, TypePath, Variant,
};

use crate::use_aliases::UseAliases;

use crate::declared::{Declared, Wrapped};
use crate::owner::Owner;
use crate::reach::{Reach, Reached, Reaching};
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct SignatureSites<'declared, 'file> {
    path: &'file Path,
    aliases: UseAliases,
    declared: &'declared Declared,
    owner: Owner,
    context: Name,
    scan: Scan,
    reach: Reach,
    found: Vec<Reached<'file>>,
}

impl<'declared, 'file> SignatureSites<'declared, 'file> {
    pub fn reached(file: &'file SourceFile, declared: &'declared Declared, reach: Reach) -> Vec<Reached<'file>> {
        let mut sites = SignatureSites {
            path: file.path(),
            aliases: UseAliases::in_file(file.syntax()),
            declared,
            owner: Owner::default(),
            context: Name::default(),
            scan: Scan::Idle,
            reach,
            found: Vec::new(),
        };
        sites.visit_file(file.syntax());
        sites.found
    }

    fn edged(&mut self, item: &ImplItemFn) {
        self.widened(|sites| sites.visit_signature(&item.sig));
        self.visit_block(&item.block);
    }

    fn inherent(&mut self, walk: Walk, item: &ItemImpl) {
        match walk {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {
                item.items
                    .iter()
                    .filter_map(Syntax::impl_type)
                    .for_each(|associated| self.visit_impl_item_type(associated));
            }
        }
    }

    fn scanned(&mut self, context: Name, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.scan, Scan::Scanning);
        self.context = context;
        visit(self);
        self.scan = outer;
    }

    fn typed(&mut self, ident: &Ident, visit: impl FnOnce(&mut Self)) {
        self.widened(|sites| sites.scanned(Name::from(ident), visit));
    }
}

impl Reaching for SignatureSites<'_, '_> {
    fn reach(&mut self) -> &mut Reach {
        &mut self.reach
    }
}

impl<'ast> Visit<'ast> for SignatureSites<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.entered(module, |sites| syn::visit::visit_item_mod(sites, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let outer = std::mem::replace(&mut self.owner, Owner::of_impl(item));
        self.inherent(Walk::inherent_impls(item), item);
        self.owner = outer;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        match self.declared.edge(&self.owner, item, self.reach) {
            Some(Wrapped::Bare) => self.visit_block(&item.block),
            Some(Wrapped::Composite) => self.edged(item),
            None => syn::visit::visit_impl_item_fn(self, item),
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match Wrapped::of_fields(&item.fields) {
            Some(Wrapped::Bare) => {}
            Some(Wrapped::Composite) => self.typed(&item.ident, |sites| syn::visit::visit_item_struct(sites, item)),
            None => self.scanned(Name::from(&item.ident), |sites| syn::visit::visit_item_struct(sites, item)),
        }
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.typed(&item.ident, |sites| syn::visit::visit_item_type(sites, item));
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.typed(&item.ident, |sites| sites.visit_type(&item.ty));
        self.visit_expr(&item.expr);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.typed(&item.ident, |sites| sites.visit_type(&item.ty));
        self.visit_expr(&item.expr);
    }

    fn visit_foreign_item_static(&mut self, item: &'ast ForeignItemStatic) {
        self.typed(&item.ident, |sites| sites.visit_type(&item.ty));
    }

    fn visit_trait_item_type(&mut self, item: &'ast TraitItemType) {
        self.typed(&item.ident, |sites| syn::visit::visit_trait_item_type(sites, item));
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        self.typed(&item.ident, |sites| sites.visit_type(&item.ty));
        item.default.iter().for_each(|(_, expr)| self.visit_expr(expr));
    }

    fn visit_impl_item_type(&mut self, item: &'ast ImplItemType) {
        self.typed(&item.ident, |sites| syn::visit::visit_impl_item_type(sites, item));
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.typed(&item.ident, |sites| sites.visit_type(&item.ty));
        self.visit_expr(&item.expr);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.expanded(invocation);
        syn::visit::visit_macro(self, invocation);
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        self.context = Name::from(&variant.ident);
        syn::visit::visit_variant(self, variant);
    }

    fn visit_field(&mut self, field: &'ast Field) {
        let context = field.ident.as_ref().map_or_else(|| self.context.clone(), Name::from);
        self.scanned(context, |sites| syn::visit::visit_field(sites, field));
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.scanned(Name::from(&signature.ident), |sites| syn::visit::visit_signature(sites, signature));
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let site = Site::new(self.path, LineNumber::from(path.span()));
        let found = self
            .scan
            .scanning()
            .and_then(|_| self.aliases.spelled(path, self.reach).or_else(|| self.declared.projected(path)))
            .map(|spelled| spelled.reached(site, self.context.clone()));
        self.found.extend(found);
        syn::visit::visit_type_path(self, path);
    }
}
