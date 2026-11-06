use syn::{Fields, PathArguments, Type, TypePath};

use lint_core::Syntax;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrapped {
    Bare,
    Composite,
}

impl Wrapped {
    pub fn of_fields(fields: &Fields) -> Option<Wrapped> {
        Syntax::newtype_field(fields).map(|field| Self::of_type(&field.ty))
    }

    fn of_type(ty: &Type) -> Wrapped {
        match ty {
            Type::Reference(reference) => Self::of_type(&reference.elem),
            Type::Paren(inner) => Self::of_type(&inner.elem),
            Type::Group(inner) => Self::of_type(&inner.elem),
            Type::Path(path) => Self::of_path(path),
            _ => Wrapped::Composite,
        }
    }

    fn of_path(path: &TypePath) -> Wrapped {
        path.path.segments.iter().find_map(|segment| Self::of_arguments(&segment.arguments)).unwrap_or(Wrapped::Bare)
    }

    fn of_arguments(arguments: &PathArguments) -> Option<Wrapped> {
        match arguments {
            PathArguments::None => None,
            PathArguments::AngleBracketed(_) | PathArguments::Parenthesized(_) => Some(Wrapped::Composite),
        }
    }
}
