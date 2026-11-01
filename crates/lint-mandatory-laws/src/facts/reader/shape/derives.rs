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
            .filter(|name| name.matches::<Whole>(&Self::SERDE))
            .collect()
    }

    fn listed(attribute: &Attribute) -> Vec<Path> {
        if attribute.path().is_ident(Self::DERIVE) {
            attribute
                .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .ok()
                .into_iter()
                .flatten()
                .collect()
        } else if attribute.path().is_ident(Self::CONDITIONAL) {
            attribute
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .ok()
                .into_iter()
                .flatten()
                .skip(1)
                .flat_map(Self::conditional)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn conditional(meta: Meta) -> Vec<Path> {
        match meta {
            Meta::List(list) if list.path.is_ident(Self::DERIVE) => list
                .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .ok()
                .into_iter()
                .flatten()
                .collect(),
            _ => Vec::new(),
        }
    }
}
