use std::collections::BTreeMap;

use syn::ItemStruct;
use syn::visit::Visit;

use lint_core::{Home, Parsed};
use lint_layout::Name;

use super::method_shape::Spelling;

#[derive(Debug, Default)]
pub struct Inners<'head> {
    by_home: BTreeMap<Home<'head>, BTreeMap<Name, Spelling>>,
}

impl<'head> From<Parsed<'_, 'head>> for Inners<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Inners<'head> {
        let mut reading = InnerTypes::default();
        reading.visit_file(parsed.file().syntax());
        Inners { by_home: BTreeMap::from([(Home::from(parsed.head()), reading.0)]) }
    }
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
struct InnerTypes(BTreeMap<Name, Spelling>);

impl<'ast> Visit<'ast> for InnerTypes {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let only = Some(&item.fields).filter(|fields| fields.len() == 1).and_then(|fields| fields.iter().next());
        self.0.extend(only.map(|field| (Name::from(&item.ident), Spelling::from(&field.ty))));
    }
}
