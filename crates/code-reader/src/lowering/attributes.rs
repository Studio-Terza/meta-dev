use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Path, Token};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
struct Element(Meta);

impl Element {
    fn path(self) -> Option<Path> {
        match self.0 {
            Meta::Path(path) => Some(path),
            Meta::List(_) | Meta::NameValue(_) => None,
        }
    }

    fn named(path: &Path, name: &[Word<'static>]) -> Option<()> {
        path.get_ident().map(Name::from).and_then(|ident| ident.matches::<Whole>(name).map(|_| ()))
    }
}

#[derive(Debug)]
pub(super) struct Attributes {
    metas: Vec<Meta>,
}

impl From<&[Attribute]> for Attributes {
    fn from(attributes: &[Attribute]) -> Attributes {
        Attributes { metas: attributes.iter().flat_map(|attribute| Self::applied(&attribute.meta)).collect() }
    }
}

impl Attributes {
    const CONDITIONAL: [Word<'static>; 1] = words!["cfg_attr"];
    const DERIVE: [Word<'static>; 1] = words!["derive"];

    pub(super) fn derived(&self) -> Vec<Path> {
        self.metas.iter().flat_map(|meta| Self::nested(meta, &Self::DERIVE)).filter_map(Element::path).collect()
    }

    fn applied(meta: &Meta) -> Vec<Meta> {
        Self::nested(meta, &Self::CONDITIONAL).split_first().map_or_else(
            || vec![meta.clone()],
            |(_, applied)| applied.iter().flat_map(|element| Self::applied(&element.0)).collect(),
        )
    }

    fn nested(meta: &Meta, name: &[Word<'static>]) -> Vec<Element> {
        meta.require_list()
            .ok()
            .and_then(|list| Element::named(&list.path, name).map(|()| list))
            .and_then(|list| list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok())
            .map_or_default(|parsed| parsed.into_iter().map(Element).collect())
    }
}
