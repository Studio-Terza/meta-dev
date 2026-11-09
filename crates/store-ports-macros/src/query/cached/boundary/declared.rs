use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Token, Type, parse_quote};

use super::unexpanded::Unexpanded;

#[derive(Debug, Deserialize)]
pub struct Column {
    name: ColumnName,
    type_info: Storage,
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
struct ColumnName {
    ident: Ident,
    nullability: Nullability,
    overridden: Option<Type>,
}

#[derive(Clone, Copy, Debug)]
enum Nullability {
    Described,
    Present,
    Nullable,
}

impl Column {
    pub fn typed(self, nullable: Nullable) -> Result<Typed, Unexpanded> {
        let ColumnName { ident, nullability, overridden } = self.name;
        let base = overridden.map_or_else(|| self.type_info.rust(&ident), Ok)?;
        Ok(Typed { ident, column: nullability.presence(nullable).wrapped(base) })
    }
}

impl Nullability {
    fn presence(self, described: Nullable) -> Presence {
        match self {
            Nullability::Present => Presence::Always,
            Nullability::Nullable => Presence::Maybe,
            Nullability::Described => described.presence(),
        }
    }

    fn parsed(input: ParseStream<'_>) -> syn::Result<Nullability> {
        let present = input.parse::<Option<Token![!]>>()?.map(|_bang| Nullability::Present);
        let nullable = input.parse::<Option<Token![?]>>()?.map(|_question| Nullability::Nullable);
        Ok(present.or(nullable).unwrap_or(Nullability::Described))
    }
}

impl Parse for ColumnName {
    fn parse(input: ParseStream<'_>) -> syn::Result<ColumnName> {
        let ident = Ident::parse_any(input)?;
        let nullability = Nullability::parsed(input)?;
        let overridden = input.parse::<Option<Token![:]>>()?.map(|_colon| input.parse::<Type>()).transpose()?;
        Ok(ColumnName { ident, nullability, overridden })
    }
}

impl TryFrom<String> for ColumnName {
    type Error = syn::Error;

    fn try_from(name: String) -> syn::Result<ColumnName> {
        syn::parse_str(&name)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "Option<bool>")]
pub enum Nullable {
    Never,
    Possible,
}

#[derive(Clone, Copy, Debug, Deserialize)]
enum Storage {
    Integer,
    Int4,
    Int64,
    Real,
    Float,
    Text,
    Blob,
    #[serde(other)]
    Unmapped,
}

#[derive(Clone, Copy, Debug)]
enum Presence {
    Always,
    Maybe,
}

#[derive(Debug)]
pub struct Typed {
    ident: Ident,
    column: Type,
}

impl Nullable {
    fn presence(self) -> Presence {
        match self {
            Nullable::Never => Presence::Always,
            Nullable::Possible => Presence::Maybe,
        }
    }
}

impl Storage {
    fn rust(self, ident: &Ident) -> Result<Type, Unexpanded> {
        match self {
            Storage::Integer | Storage::Int4 | Storage::Int64 => Ok(parse_quote!(i64)),
            Storage::Real | Storage::Float => Ok(parse_quote!(f64)),
            Storage::Text => Ok(parse_quote!(::std::string::String)),
            Storage::Blob => Ok(parse_quote!(::std::vec::Vec<u8>)),
            Storage::Unmapped => Err(Unexpanded::Unmapped(ident.clone())),
        }
    }
}

impl Presence {
    fn wrapped(self, base: Type) -> Type {
        match self {
            Presence::Always => base,
            Presence::Maybe => parse_quote!(::core::option::Option<#base>),
        }
    }
}

impl From<Option<bool>> for Nullable {
    fn from(described: Option<bool>) -> Nullable {
        described
            .and_then(|nullable| nullable.then_some(Nullable::Possible).or(Some(Nullable::Never)))
            .unwrap_or(Nullable::Possible)
    }
}

impl ToTokens for Typed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Typed { ident, column } = self;
        let decoded = quote!(::store_ports::FromValue);
        tokens.extend(quote! {
            #ident: ::core::convert::Into::into(<#column as #decoded>::from_value(values.taken()?)?),
        });
    }
}
