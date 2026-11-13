use std::collections::BTreeMap;

use syn::{ItemEnum, ItemMod, ItemStruct};

use lint_layout::Name;

use crate::declared_type::DeclaredType;
use crate::home::Home;
use crate::source_file::Parsed;
use crate::walk::Walk;
use crate::watch::{Descend, Kind, Watch};

#[derive(Debug)]
pub struct TypeHomes<'file> {
    declared: BTreeMap<Name, Vec<Home<'file>>>,
    reading: Home<'file>,
}

impl<'file> TypeHomes<'file> {
    #[must_use]
    pub fn resolve(&self, name: Name, from: Home<'file>) -> DeclaredType<'file> {
        DeclaredType::new(self.home_of(&name, from), name)
    }

    #[must_use]
    pub fn home_of(&self, name: &Name, from: Home<'file>) -> Home<'file> {
        let candidates = self.declared.get(name).map_or(&[][..], Vec::as_slice);
        candidates
            .iter()
            .find(|home| **home == from)
            .or_else(|| candidates.iter().find(|home| home.in_crate_with(from).is_some()))
            .or_else(|| candidates.first())
            .map_or(from, |home| *home)
    }

    fn absorbed(self, part: TypeHomes<'file>) -> TypeHomes<'file> {
        let placed = part
            .declared
            .into_iter()
            .flat_map(|(name, homes)| homes.into_iter().map(move |home| DeclaredType::new(home, name.clone())));
        placed.fold(self, |merged, declared| {
            let mut reading = TypeHomes { reading: declared.home(), ..merged };
            reading.declare(declared.as_ref().clone());
            reading
        })
    }

    fn declared(&mut self, attributes: &[syn::Attribute], ident: &syn::Ident) -> Descend {
        let entered = Descend::from(Walk::outside_tests(attributes)).entered();
        entered.map(|_| Name::from(ident)).into_iter().for_each(|name| self.declare(name));
        Descend::Over
    }

    fn declare(&mut self, name: Name) {
        let known = self.declared.entry(name).or_default();
        let fresh = known.iter().find(|home| **home == self.reading).map_or(Some(self.reading), |_| None);
        known.extend(fresh);
    }
}

impl<'ast> Watch<'ast> for TypeHomes<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemStruct, Kind::ItemEnum]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        self.declared(&item.attrs, &item.ident)
    }
}

impl<'head> From<Parsed<'_, 'head>> for TypeHomes<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> TypeHomes<'head> {
        TypeHomes { declared: BTreeMap::new(), reading: Home::from(parsed.head()) }
    }
}

impl<'head> FromIterator<TypeHomes<'head>> for TypeHomes<'head> {
    fn from_iter<Parts: IntoIterator<Item = TypeHomes<'head>>>(parts: Parts) -> TypeHomes<'head> {
        let merged = parts
            .into_iter()
            .fold(TypeHomes { declared: BTreeMap::new(), reading: Home::NOWHERE }, TypeHomes::absorbed);
        TypeHomes { reading: Home::NOWHERE, ..merged }
    }
}
