use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Path, Token};

use lint_layout::Whole;
use lint_layout::{Name, Word, words};

#[derive(Debug)]
pub(crate) struct Derives;

impl Derives {
    const DERIVE: &'static str = "derive";
    const CONDITIONAL: &'static str = "cfg_attr";
    const SERDE: [Word<'static>; 2] = words!["Serialize", "Deserialize"];

    pub(crate) fn named(attributes: &[Attribute]) -> Vec<Name> {
        attributes
            .iter()
            .flat_map(Self::listed)
            .filter_map(|path| path.segments.last().map(|last| Name::from(&last.ident)))
            .filter(|name| name.matches::<Whole>(&Self::SERDE).is_some())
            .collect()
    }

    fn listed(attribute: &Attribute) -> Vec<Path> {
        match Listing::from(attribute.path()) {
            Listing::Derive => attribute
                .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .ok()
                .into_iter()
                .flatten()
                .collect(),
            Listing::Conditional => attribute
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok()
                .into_iter()
                .flatten()
                .skip(1)
                .flat_map(|meta| Self::conditional(&meta))
                .collect(),
            Listing::Other => Vec::new(),
        }
    }

    fn conditional(meta: &Meta) -> Vec<Path> {
        meta.require_list()
            .ok()
            .filter(|list| list.path.is_ident(Self::DERIVE))
            .and_then(|list| list.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .into_iter()
            .flatten()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Listing {
    Derive,
    Conditional,
    Other,
}

impl Listing {
    const NAMED: [(&'static str, Listing); 2] =
        [(Derives::DERIVE, Listing::Derive), (Derives::CONDITIONAL, Listing::Conditional)];
}

impl From<&Path> for Listing {
    fn from(path: &Path) -> Listing {
        Listing::NAMED.iter().find(|(name, _)| path.is_ident(name)).map_or(Listing::Other, |(_, listing)| *listing)
    }
}
