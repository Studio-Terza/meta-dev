mod scan;
mod spelling;
mod use_aliases;

use std::path::PathBuf;

use scan::Scan;
use spelling::Spelling;
use use_aliases::UseAliases;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Field, Fields, ImplItemFn, ItemImpl, ItemMod, ItemStruct, Signature, TypePath, Variant};

use crate::finding::Finding;
use crate::newtypes::Newtypes;
use crate::text_sites::Owner;
use lint_core::Gate;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

pub struct SignatureSites<'newtypes> {
    path: PathBuf,
    aliases: UseAliases,
    newtypes: &'newtypes Newtypes,
    owner: Owner,
    context: Name,
    scan: Scan,
    found: Vec<Violation>,
}

impl<'newtypes> SignatureSites<'newtypes> {
    const NEWTYPE_EDGES: [Word<'static>; 3] = words!["new", "try_new", "into_inner"];

    pub fn violations(file: &SourceFile, newtypes: &'newtypes Newtypes) -> Vec<Violation> {
        let mut sites = SignatureSites {
            path: file.path().to_owned(),
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

    fn scanned(&mut self, context: Name, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.scan, Scan::Scanning);
        self.context = context;
        visit(self);
        self.scan = outer;
    }

    fn newtype_edge(&self, item: &ImplItemFn) -> Option<Name> {
        let name = Name::from(&item.sig.ident);
        self.newtypes
            .holds(&self.owner)
            .and(name.matches::<Whole>(&Self::NEWTYPE_EDGES))
            .filter(|_| item.sig.inputs.len() == 1)
            .map(|_| name)
    }
}

impl<'ast> Visit<'ast> for SignatureSites<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered if Gate::BOUNDARY.names(module).is_none() => syn::visit::visit_item_mod(self, module),
            Walk::Entered | Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => {
                let outer = std::mem::replace(&mut self.owner, Owner::of_impl(item));
                syn::visit::visit_item_impl(self, item);
                self.owner = outer;
            }
            Walk::Skipped => {}
        }
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        match self.newtype_edge(item) {
            Some(_) => self.visit_block(&item.block),
            None => syn::visit::visit_impl_item_fn(self, item),
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        match &item.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {}
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {
                self.scanned(Name::from(&item.ident), |sites| syn::visit::visit_generics(sites, &item.generics));
                item.fields.iter().for_each(|field| self.visit_field(field));
            }
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
        let found = match self.scan {
            Scan::Scanning => self.finding(path),
            Scan::Idle => None,
        };
        self.found
            .extend(found.map(|finding| Violation::new(Site::new(&self.path, LineNumber::from(path.span())), finding)));
        syn::visit::visit_type_path(self, path);
    }
}
