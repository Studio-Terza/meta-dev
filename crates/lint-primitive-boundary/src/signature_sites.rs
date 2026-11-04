mod scan;
mod spelling;
mod use_aliases;

use std::path::Path;

use scan::Scan;
use spelling::Spelling;
use use_aliases::UseAliases;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Field, ImplItemFn, ItemImpl, ItemMod, ItemStruct, Signature, TypePath, Variant};

use crate::finding::Finding;
use crate::newtypes::Newtypes;
use crate::owner::Owner;
use lint_core::Gate;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

pub struct SignatureSites<'newtypes, 'file> {
    path: &'file Path,
    aliases: UseAliases,
    newtypes: &'newtypes Newtypes,
    owner: Owner,
    context: Name,
    scan: Scan,
    found: Vec<Violation<'file>>,
}

impl<'newtypes, 'file> SignatureSites<'newtypes, 'file> {
    pub fn violations(file: &'file SourceFile, newtypes: &'newtypes Newtypes) -> Vec<Violation<'file>> {
        let mut sites = SignatureSites {
            path: file.path(),
            aliases: UseAliases::in_file(file.syntax()),
            newtypes,
            owner: Owner::default(),
            context: Name::default(),
            scan: Scan::Idle,
            found: Vec::new(),
        };
        sites.visit_file(file.syntax());
        sites.found
    }

    fn finding(&self, path: &TypePath) -> Option<Finding> {
        Some(path)
            .filter(|path| path.qself.is_none())
            .and_then(|path| Spelling::read(&self.aliases.resolve(&path.path)))
            .map(|spelling| spelling.into_finding(self.context.clone()))
    }

    fn inherent(&mut self, walk: Walk, item: &ItemImpl) {
        match walk {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn declared(&mut self, item: &ItemStruct) {
        self.scanned(Name::from(&item.ident), |sites| syn::visit::visit_generics(sites, &item.generics));
        item.fields.iter().for_each(|field| self.visit_field(field));
    }

    fn scanned(&mut self, context: Name, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.scan, Scan::Scanning);
        self.context = context;
        visit(self);
        self.scan = outer;
    }
}

impl<'ast> Visit<'ast> for SignatureSites<'_, '_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::BOUNDARY.enter_outside(module, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let outer = std::mem::replace(&mut self.owner, Owner::of_impl(item));
        self.inherent(Walk::inherent_impls(item), item);
        self.owner = outer;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        match self.newtypes.edge(&self.owner, item) {
            Some(_) => self.visit_block(&item.block),
            None => syn::visit::visit_impl_item_fn(self, item),
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match Syntax::newtype_field(&item.fields) {
            Some(_) => {}
            None => self.declared(item),
        }
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
        let found = Some(self.scan).filter(|scan| *scan == Scan::Scanning).and_then(|_| self.finding(path));
        self.found
            .extend(found.map(|finding| Violation::new(Site::new(self.path, LineNumber::from(path.span())), finding)));
        syn::visit::visit_type_path(self, path);
    }
}
