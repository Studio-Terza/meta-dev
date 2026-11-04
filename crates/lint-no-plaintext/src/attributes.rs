use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Path, Token};

use lint_layout::{Name, Word};

pub struct Attributes {
    metas: Vec<Meta>,
}

impl Attributes {
    const CONDITIONAL: Word<'static> = Word::new("cfg_attr");
    const DERIVE: Word<'static> = Word::new("derive");
    const SERDE: Word<'static> = Word::new("serde");
    const DESERIALIZE: &'static str = "Deserialize";
    const VALIDATING: &'static str = "try_from";

    pub fn flattened_from(attributes: &[Attribute]) -> Attributes {
        Attributes { metas: attributes.iter().flat_map(|attribute| Self::flattened(&attribute.meta)).collect() }
    }

    pub fn deserialize_derive(&self) -> Option<Path> {
        self.metas
            .iter()
            .flat_map(|meta| Self::nested(meta, Self::DERIVE))
            .filter_map(Self::path_of)
            .find(|path| path.segments.last().is_some_and(|last| last.ident == Self::DESERIALIZE))
    }

    pub fn validation(&self) -> Option<Meta> {
        self.metas
            .iter()
            .flat_map(|meta| Self::nested(meta, Self::SERDE))
            .find(|setting| matches!(setting, Meta::NameValue(pair) if pair.path.is_ident(Self::VALIDATING)))
    }

    fn flattened(meta: &Meta) -> Vec<Meta> {
        Self::nested(meta, Self::CONDITIONAL)
            .split_first()
            .map_or_else(|| vec![meta.clone()], |(_, rest)| rest.to_vec())
    }

    fn nested(meta: &Meta, name: Word<'_>) -> Vec<Meta> {
        meta.require_list()
            .ok()
            .filter(|list| list.path.get_ident().is_some_and(|ident| Name::from(ident) == name))
            .and_then(|list| list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok())
            .map(|parsed| parsed.into_iter().collect())
            .unwrap_or_default()
    }

    fn path_of(meta: Meta) -> Option<Path> {
        match meta {
            Meta::Path(path) => Some(path),
            Meta::List(_) | Meta::NameValue(_) => None,
        }
    }
}
