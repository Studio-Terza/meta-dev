use syntax::parse::{Parse, ParseStream};
use syntax::{Ident, IdentExt, Span, ToTokens, Token, TokenStream, Type, quote};

use query_cache::Described;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct ColumnName {
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

#[derive(Debug, thiserror::Error)]
#[error("column {0} is described with a type that maps to no Rust type; name its type, as in \"{0}!: String\"")]
struct Unmapped(Ident);

impl Typed {
    pub fn described(described: &Described<ColumnName>, span: Span) -> syntax::Result<Typed> {
        let ColumnName { ident, nullability, overridden } = described.as_ref();
        let spelled = described.storage().spelled().map(syntax::parse::parse_str::<Type>).transpose()?;
        let base = overridden.clone().or(spelled).ok_or_else(|| syntax::Error::new(span, Unmapped(ident.clone())))?;
        let presence = nullability.presence(described);
        Ok(Typed { ident: ident.clone(), column: presence.wrapped(base) })
    }
}

impl Nullability {
    fn presence(self, described: &Described<ColumnName>) -> Presence {
        match self {
            Nullability::Present => Presence::Always,
            Nullability::Nullable => Presence::Maybe,
            Nullability::Described => described.nullable().possible().map_or(Presence::Always, |()| Presence::Maybe),
        }
    }

    fn parsed(input: ParseStream<'_>) -> syntax::Result<Nullability> {
        let present = input.parse::<Option<Token![!]>>()?.map(|_bang| Nullability::Present);
        let nullable = input.parse::<Option<Token![?]>>()?.map(|_question| Nullability::Nullable);
        Ok(present.or(nullable).unwrap_or(Nullability::Described))
    }
}

impl Parse for ColumnName {
    fn parse(input: ParseStream<'_>) -> syntax::Result<ColumnName> {
        let ident = Ident::parse_any(input)?;
        let nullability = Nullability::parsed(input)?;
        let overridden = input.parse::<Option<Token![:]>>()?.map(|_colon| input.parse::<Type>()).transpose()?;
        Ok(ColumnName { ident, nullability, overridden })
    }
}

impl TryFrom<String> for ColumnName {
    type Error = syntax::Error;

    fn try_from(name: String) -> syntax::Result<ColumnName> {
        syntax::parse::parse_str(&name)
    }
}

impl Presence {
    fn wrapped(self, base: Type) -> Type {
        match self {
            Presence::Always => base,
            Presence::Maybe => syntax::parse::parse_quote!(::core::option::Option<#base>),
        }
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
