use syn::{ReturnType, Type, TypeTuple};

use lint_core::TypeName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Returns<'found> {
    Vertex(&'found TypeName),
    TupleHoldingVertex,
    Other,
}

impl<'found> Returns<'found> {
    pub fn classify(output: &ReturnType, vertex: impl Fn(&Type) -> Option<&'found TypeName>) -> Returns<'found> {
        match output {
            ReturnType::Default => Returns::Other,
            ReturnType::Type(_, ty) => match &**ty {
                Type::Reference(_) => Returns::Other,
                Type::Tuple(tuple) => Self::of_tuple(tuple, vertex),
                other => vertex(other).map_or(Returns::Other, Returns::Vertex),
            },
        }
    }

    pub fn vertex(&self) -> Option<&'found TypeName> {
        match self {
            Returns::Vertex(name) => Some(name),
            Returns::TupleHoldingVertex | Returns::Other => None,
        }
    }

    fn of_tuple(tuple: &TypeTuple, vertex: impl Fn(&Type) -> Option<&'found TypeName>) -> Returns<'found> {
        if tuple.elems.iter().any(|element| vertex(element).is_some()) {
            Returns::TupleHoldingVertex
        } else {
            Returns::Other
        }
    }
}
