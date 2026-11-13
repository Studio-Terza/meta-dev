use std::collections::BTreeMap;

use syn::ItemStruct;

use lint_core::Home;
use lint_layout::Name;
use lint_sweep::{Descend, Kind, Watch};

use super::method_shape::Spelling;

#[derive(Debug, Default)]
pub struct Inners<'head> {
    by_home: BTreeMap<Home<'head>, BTreeMap<Name, Spelling>>,
}

impl<'head> FromIterator<Inners<'head>> for Inners<'head> {
    fn from_iter<Parts: IntoIterator<Item = Inners<'head>>>(parts: Parts) -> Inners<'head> {
        let by_home = parts.into_iter().flat_map(|part| part.by_home).fold(
            BTreeMap::new(),
            |mut merged: BTreeMap<Home<'head>, BTreeMap<Name, Spelling>>, (home, inners)| {
                merged.entry(home).or_default().extend(inners);
                merged
            },
        );
        Inners { by_home }
    }
}

impl<'head> Inners<'head> {
    pub fn inner_of(&self, home: &Home<'head>, name: &Name) -> Option<&Spelling> {
        self.by_home.get(home)?.get(name)
    }
}

#[derive(Debug, Default)]
pub struct InnerTypes(BTreeMap<Name, Spelling>);

impl InnerTypes {
    pub fn placed(self, home: Option<Home<'_>>) -> Inners<'_> {
        Inners { by_home: home.map(|placed| (placed, self.0)).into_iter().collect() }
    }
}

impl<'ast> Watch<'ast> for InnerTypes {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemStruct]
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        let only = Some(&item.fields).filter(|fields| fields.len() == 1).and_then(|fields| fields.iter().next());
        self.0.extend(only.map(|field| (Name::from(&item.ident), Spelling::from(&field.ty))));
        Descend::Over
    }
}
