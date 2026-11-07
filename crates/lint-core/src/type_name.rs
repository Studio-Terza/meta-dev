use std::fmt;

use quote::ToTokens;
use syn::{Ident, Type};

use crate::syntax::Syntax;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeName(String);

impl TypeName {
    const WRAPPERS: [&'static str; 2] = ["Option", "Result"];
    const SELF: &'static str = "Self";

    #[must_use]
    pub fn from_type(ty: &Type) -> Self {
        Self(Self::unwrapped(ty).to_token_stream().to_string().chars().filter(|letter| *letter != ' ').collect())
    }

    #[must_use]
    pub fn named(ident: &Ident) -> Self {
        Self(ident.to_string())
    }

    #[must_use]
    pub fn or_self<'name>(&'name self, self_type: &'name Self) -> &'name Self {
        Some(self).filter(|name| name.0 == Self::SELF).map_or(self, |_| self_type)
    }

    fn unwrapped(ty: &Type) -> &Type {
        Syntax::type_path(ty)
            .and_then(|path| path.path.segments.last())
            .filter(|segment| Self::WRAPPERS.iter().any(|wrapper| segment.ident == wrapper))
            .and_then(|segment| Syntax::angled(&segment.arguments))
            .and_then(|arguments| arguments.args.first())
            .and_then(Syntax::type_argument)
            .unwrap_or(ty)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
