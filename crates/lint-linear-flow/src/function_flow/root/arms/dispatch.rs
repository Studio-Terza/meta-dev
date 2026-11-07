use syn::Pat;

use crate::finding::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dispatch {
    Variant,
    Value,
}

impl From<&Pat> for Dispatch {
    fn from(pattern: &Pat) -> Self {
        match pattern {
            Pat::Lit(_) | Pat::Range(_) | Pat::Tuple(_) | Pat::Slice(_) | Pat::Const(_) => Self::Value,
            Pat::Or(alternatives) => Self::worst(alternatives.cases.iter().map(Self::from)),
            Pat::Reference(reference) => Self::from(&*reference.pat),
            Pat::Paren(inner) => Self::from(&*inner.pat),
            Pat::Ident(binding) => binding.subpat.as_ref().map_or(Self::Variant, |(_, inner)| Self::from(&**inner)),
            Pat::TupleStruct(variant) => Self::worst(variant.elems.iter().map(Self::inside)),
            Pat::Struct(variant) => Self::worst(variant.fields.iter().map(|field| Self::inside(&field.pat))),
            _ => Self::Variant,
        }
    }
}

impl Dispatch {
    pub fn fault(self) -> Option<Finding> {
        match self {
            Self::Variant => None,
            Self::Value => Some(Finding::ValuePattern),
        }
    }

    fn inside(pattern: &Pat) -> Self {
        match pattern {
            Pat::Tuple(tuple) => Self::worst(tuple.elems.iter().map(Self::inside)),
            Pat::Slice(slice) => Self::worst(slice.elems.iter().map(Self::inside)),
            other => Self::from(other),
        }
    }

    fn worst(dispatches: impl Iterator<Item = Self>) -> Self {
        dispatches.max().unwrap_or(Self::Variant)
    }
}
