use std::fmt;

use quote::ToTokens;
use syn::{GenericArgument, Ident, PathArguments, Type};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeName(String);

impl TypeName {
    const WRAPPERS: [&'static str; 2] = ["Option", "Result"];
    const SELF: &'static str = "Self";

    #[must_use]
    pub fn from_type(ty: &Type) -> TypeName {
        TypeName(Self::unwrapped(ty).to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect())
    }

    #[must_use]
    pub fn named(ident: &Ident) -> TypeName {
        TypeName(ident.to_string())
    }

    #[must_use]
    pub fn or_self(self, self_type: &TypeName) -> TypeName {
        if self.0 == Self::SELF { self_type.clone() } else { self }
    }

    fn unwrapped(ty: &Type) -> &Type {
        let last = match ty {
            Type::Path(path) => path.path.segments.last(),
            _ => None,
        };
        last.filter(|segment| Self::WRAPPERS.iter().any(|wrapper| segment.ident == wrapper))
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.first(),
                _ => None,
            })
            .and_then(|argument| match argument {
                GenericArgument::Type(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(ty)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
