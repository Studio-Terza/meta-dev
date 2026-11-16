use syntax::{ReturnType, Type};

use lint_core::TypeName;

#[derive(Debug)]
pub enum WrittenReturns {
    Other,
    Single(TypeName),
    Tuple(Vec<TypeName>),
}

impl From<&ReturnType> for WrittenReturns {
    fn from(output: &ReturnType) -> WrittenReturns {
        match output {
            ReturnType::Default => WrittenReturns::Other,
            ReturnType::Type(_, ty) => WrittenReturns::from(ty.as_ref()),
        }
    }
}

impl From<&Type> for WrittenReturns {
    fn from(ty: &Type) -> WrittenReturns {
        match ty {
            Type::Reference(_) => WrittenReturns::Other,
            Type::Tuple(tuple) => WrittenReturns::Tuple(tuple.elems.iter().map(TypeName::from_type).collect()),
            other => WrittenReturns::Single(TypeName::from_type(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Returns<'found> {
    Vertex(&'found TypeName),
    TupleHoldingVertex,
    Other,
}

impl<'found> Returns<'found> {
    pub fn resolved(
        written: &WrittenReturns,
        vertex: impl Fn(&TypeName) -> Option<&'found TypeName>,
    ) -> Returns<'found> {
        match written {
            WrittenReturns::Other => Returns::Other,
            WrittenReturns::Single(name) => vertex(name).map_or(Returns::Other, Returns::Vertex),
            WrittenReturns::Tuple(names) => {
                names.iter().find_map(vertex).map_or(Returns::Other, |_| Returns::TupleHoldingVertex)
            }
        }
    }

    pub fn vertex(&self) -> Option<&'found TypeName> {
        match self {
            Returns::Vertex(name) => Some(name),
            Returns::TupleHoldingVertex | Returns::Other => None,
        }
    }
}
