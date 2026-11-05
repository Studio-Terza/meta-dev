use syn::Pat;

use crate::function_flow::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dispatch {
    Variant,
    Value,
}

impl From<&Pat> for Dispatch {
    fn from(pattern: &Pat) -> Dispatch {
        match pattern {
            Pat::Lit(_) | Pat::Range(_) | Pat::Tuple(_) | Pat::Slice(_) | Pat::Const(_) => Dispatch::Value,
            Pat::Or(alternatives) => Self::worst(alternatives.cases.iter().map(Dispatch::from)),
            Pat::Reference(reference) => Dispatch::from(&*reference.pat),
            Pat::Paren(inner) => Dispatch::from(&*inner.pat),
            Pat::Ident(binding) => {
                binding.subpat.as_ref().map_or(Dispatch::Variant, |(_, inner)| Dispatch::from(&**inner))
            }
            Pat::TupleStruct(variant) => Self::worst(variant.elems.iter().map(Self::inside)),
            Pat::Struct(variant) => Self::worst(variant.fields.iter().map(|field| Self::inside(&field.pat))),
            _ => Dispatch::Variant,
        }
    }
}

impl Dispatch {
    pub fn fault(self) -> Option<Finding> {
        match self {
            Dispatch::Variant => None,
            Dispatch::Value => Some(Finding::ValuePattern),
        }
    }

    fn inside(pattern: &Pat) -> Dispatch {
        match pattern {
            Pat::Tuple(tuple) => Self::worst(tuple.elems.iter().map(Self::inside)),
            Pat::Slice(slice) => Self::worst(slice.elems.iter().map(Self::inside)),
            other => Dispatch::from(other),
        }
    }

    fn worst(dispatches: impl Iterator<Item = Dispatch>) -> Dispatch {
        dispatches.max().unwrap_or(Dispatch::Variant)
    }
}
