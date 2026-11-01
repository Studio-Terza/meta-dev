use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token};

pub struct Attributes {
    metas: Vec<Meta>,
}

impl Attributes {
    const CONDITIONAL: &'static str = "cfg_attr";
    const DERIVE: &'static str = "derive";
    const SERDE: &'static str = "serde";
    const DESERIALIZE: &'static str = "Deserialize";
    const VALIDATING: &'static str = "try_from";

    pub fn flattened_from(attributes: &[Attribute]) -> Attributes {
        Attributes { metas: attributes.iter().flat_map(|attribute| Self::flattened(&attribute.meta)).collect() }
    }

    pub fn derives_deserialize(&self) -> bool {
        self.metas.iter().flat_map(|meta| Self::nested(meta, Self::DERIVE)).any(
            |derived| matches!(derived, Meta::Path(path) if path.segments.last().is_some_and(|last| last.ident == Self::DESERIALIZE)),
        )
    }

    pub fn validation(&self) -> Option<Meta> {
        self.metas
            .iter()
            .flat_map(|meta| Self::nested(meta, Self::SERDE))
            .find(|setting| matches!(setting, Meta::NameValue(pair) if pair.path.is_ident(Self::VALIDATING)))
    }

    fn flattened(meta: &Meta) -> Vec<Meta> {
        let inner = Self::nested(meta, Self::CONDITIONAL);
        if inner.is_empty() { vec![meta.clone()] } else { inner.into_iter().skip(1).collect() }
    }

    fn nested(meta: &Meta, name: &str) -> Vec<Meta> {
        match meta {
            Meta::List(list) if list.path.is_ident(name) => list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map(|parsed| parsed.into_iter().collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }
}
