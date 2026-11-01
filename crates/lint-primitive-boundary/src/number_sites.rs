mod rank;
mod sighting;
mod sightings;
mod use_aliases;

use std::path::PathBuf;

use rank::Rank;
use sighting::Sighting;
use sightings::Sightings;
pub use use_aliases::UseAliases;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, ConstParam, Field, Fields, ImplItemConst, ItemConst, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, Pat,
    PatType, Signature, TraitItemConst, TypePath,
};

use super::finding::Finding;
use lint_core::Gate;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::UsePath;
use lint_layout::{LoweredInfix, Name, Whole, Word, words};

pub struct NumberSites {
    path: PathBuf,
    aliases: UseAliases,
    context: Name,
    parameter: Option<Name>,
    newtypes: Vec<Name>,
    sightings: Sightings,
}

impl NumberSites {
    const PRIMITIVES: [Word<'static>; 12] =
        words!["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64"];
    const SIZES: [Word<'static>; 2] = words!["usize", "isize"];
    const WIRE_TYPES: [&'static [Word<'static>]; 9] = [
        &words!["serde_json", "Value"],
        &words!["toml", "Value"],
        &words!["serde_yaml", "Value"],
        &words!["rust_decimal", "Decimal"],
        &words!["bigdecimal", "BigDecimal"],
        &words!["num_bigint", "BigInt"],
        &words!["num_bigint", "BigUint"],
        &words!["std", "any", "Any"],
        &words!["core", "any", "Any"],
    ];
    const SIZE_WORDS: [Word<'static>; 6] = words!["length", "len", "capacity", "size", "bytes", "index"];

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut sites = NumberSites {
            path: file.path().to_owned(),
            aliases: UseAliases::in_file(file.syntax()),
            context: Name::default(),
            parameter: None,
            newtypes: Vec::new(),
            sightings: Sightings::default(),
        };
        sites.visit_file(file.syntax());
        if sites.newtypes.is_empty() { sites.sightings.into_violations() } else { Vec::new() }
    }

    fn is_wire(path: &UsePath) -> bool {
        Self::WIRE_TYPES.iter().any(|wire| path.spells(wire))
    }

    fn is_named(path: &TypePath, names: &[Word<'_>]) -> bool {
        path.qself.is_none() && path.path.get_ident().is_some_and(|ident| Name::from(ident).matches::<Whole>(names))
    }

    fn newtype(&self, item: &ItemStruct) -> Option<Name> {
        match &item.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields
                .unnamed
                .first()
                .filter(|field| matches!(&field.ty, syn::Type::Path(path) if Self::is_named(path, &Self::PRIMITIVES) || Self::is_named(path, &Self::SIZES) || Self::is_wire(&self.aliases.resolve(&path.path))))
                .map(|_| Name::from(&item.ident)),
            _ => None,
        }
    }

    fn sighting(&self, path: &TypePath) -> Option<Sighting> {
        let wire = self.aliases.resolve(&path.path);
        let sized_by = self.parameter.as_ref().unwrap_or(&self.context);
        let site = Site::new(&self.path, LineNumber::from(path.path.span()));
        if Self::is_named(path, &Self::PRIMITIVES) {
            Some(Sighting::new(
                Rank::Number,
                Violation::new(
                    site,
                    Finding::BarePrimitive {
                        primitive: path.path.get_ident().map(Name::from).unwrap_or_default(),
                        context: self.context.clone(),
                    },
                ),
            ))
        } else if Self::is_wire(&wire) {
            Some(Sighting::new(
                Rank::Number,
                Violation::new(site, Finding::WireForm { wire, context: self.context.clone() }),
            ))
        } else if Self::is_named(path, &Self::SIZES) && !sized_by.matches::<LoweredInfix>(&Self::SIZE_WORDS) {
            Some(Sighting::new(Rank::Size, Violation::new(site, Finding::UnnamedSize { context: sized_by.clone() })))
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for NumberSites {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered if !Gate::BITS.names(module) => syn::visit::visit_item_mod(self, module),
            Walk::Entered | Walk::Skipped => {}
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match Walk::inherent_impls(item) {
            Walk::Entered => syn::visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.newtypes.extend(self.newtype(item));
        self.context = Name::from(&item.ident);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_item_enum(self, item);
    }

    fn visit_const_param(&mut self, _: &'ast ConstParam) {}

    fn visit_block(&mut self, _: &'ast Block) {}

    fn visit_field(&mut self, field: &'ast Field) {
        self.context = field.ident.as_ref().map_or_else(|| self.context.clone(), Name::from);
        syn::visit::visit_type(self, &field.ty);
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.context = Name::from(&item.ident);
        syn::visit::visit_type(self, &item.ty);
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.context = Name::from(&signature.ident);
        syn::visit::visit_signature(self, signature);
    }

    fn visit_pat_type(&mut self, typed: &'ast PatType) {
        let outer = self.parameter.take();
        self.parameter = match &*typed.pat {
            Pat::Ident(bound) => Some(Name::from(&bound.ident)),
            _ => None,
        };
        syn::visit::visit_type(self, &typed.ty);
        self.parameter = outer;
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        let sighting = self.sighting(path);
        self.sightings.file(sighting);
        syn::visit::visit_type_path(self, path);
    }
}
