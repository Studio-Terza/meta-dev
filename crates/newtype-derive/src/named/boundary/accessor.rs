use proc_macro2::TokenStream;
use quote::quote;
use reserved_names::Reserved;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Ident, Lit, Token, Type};

use super::VariantName;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Literal {
    Text,
    Integer,
    Octet,
    Byte,
    Boolean,
    Character,
}

impl Literal {
    const PATHS: [&'static str; 5] =
        ["", "core::primitive::", "std::primitive::", "::core::primitive::", "::std::primitive::"];

    fn spelled(self, head: &str, tail: &str, ty: &Type) -> Option<Literal> {
        Self::PATHS.iter().find_map(|prefix| {
            syn::parse_str::<Type>(&format!("{head}{prefix}{tail}")).ok().filter(|parsed| parsed == ty).map(|_| self)
        })
    }

    const fn admitted(self) -> &'static [Literal] {
        match self {
            Literal::Text => &[Literal::Text],
            Literal::Integer => &[Literal::Integer],
            Literal::Octet => &[Literal::Integer, Literal::Byte],
            Literal::Byte => &[Literal::Byte],
            Literal::Boolean => &[Literal::Boolean],
            Literal::Character => &[Literal::Character],
        }
    }

    const SPELLINGS: [(&'static str, &'static str, Literal); 15] = [
        ("&'static ", "str", Literal::Text),
        ("", "bool", Literal::Boolean),
        ("", "char", Literal::Character),
        ("", "u8", Literal::Octet),
        ("", "u16", Literal::Integer),
        ("", "u32", Literal::Integer),
        ("", "u64", Literal::Integer),
        ("", "u128", Literal::Integer),
        ("", "usize", Literal::Integer),
        ("", "i8", Literal::Integer),
        ("", "i16", Literal::Integer),
        ("", "i32", Literal::Integer),
        ("", "i64", Literal::Integer),
        ("", "i128", Literal::Integer),
        ("", "isize", Literal::Integer),
    ];
}

impl TryFrom<&Type> for Literal {
    type Error = Error;

    fn try_from(ty: &Type) -> syn::Result<Literal> {
        Self::SPELLINGS
            .iter()
            .find_map(|(head, tail, literal)| literal.spelled(head, tail, ty))
            .ok_or_else(|| Error::new_spanned(ty, Accessor::UNSUPPORTED))
    }
}

impl TryFrom<&Lit> for Literal {
    type Error = Error;

    fn try_from(lit: &Lit) -> syn::Result<Literal> {
        match lit {
            Lit::Str(_) => Ok(Literal::Text),
            Lit::Int(_) => Ok(Literal::Integer),
            Lit::Byte(_) => Ok(Literal::Byte),
            Lit::Bool(_) => Ok(Literal::Boolean),
            Lit::Char(_) => Ok(Literal::Character),
            _ => Err(Error::new_spanned(lit, Accessor::WRONG_TYPE)),
        }
    }
}

pub struct Accessor {
    name: Ident,
    key: String,
    ty: Type,
    literal: Literal,
}

impl Accessor {
    const UNSUPPORTED: &'static str = "an accessor type is &'static str, an integer type, bool or char";
    const WRONG_TYPE: &'static str = "this literal does not have the accessor's declared type";
    pub const ATTRIBUTE: &'static str = "accessor";
    const REPEATED: &'static str = "two accessors have this name";

    pub fn declared(attrs: &[Attribute]) -> syn::Result<Vec<Accessor>> {
        let accessors = attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::ATTRIBUTE))
            .map(Attribute::parse_args::<Accessor>)
            .collect::<syn::Result<Vec<Accessor>>>()?;
        accessors
            .iter()
            .enumerate()
            .find_map(|(index, accessor)| accessor.refusal(accessors.get(..index).unwrap_or_default()))
            .map_or(Ok(accessors), Err)
    }

    fn refusal(&self, earlier: &[Accessor]) -> Option<Error> {
        Reserved::owner(&self.key)
            .map(|_| Reserved::message())
            .or_else(|| earlier.iter().any(|accessor| accessor.key == self.key).then(|| Self::REPEATED.to_owned()))
            .map(|message| Error::new_spanned(&self.name, message))
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn checked<'lit>(&self, lit: &'lit Lit) -> syn::Result<&'lit Lit> {
        Literal::try_from(lit)
            .and_then(|found| {
                Some(found)
                    .filter(|found| self.literal.admitted().contains(found))
                    .ok_or_else(|| Error::new_spanned(lit, Self::WRONG_TYPE))
            })
            .map(|_| lit)
    }

    pub fn method(&self, owner: &Ident, names: &[VariantName]) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let arms = names.iter().map(|variant| variant.accessor_arm(owner, self));
        quote! {
            #[must_use]
            pub const fn #name(self) -> #ty {
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}

impl Parse for Accessor {
    fn parse(input: ParseStream) -> syn::Result<Accessor> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        let literal = Literal::try_from(&ty)?;
        let key = name.unraw().to_string();
        Ok(Accessor { name, key, ty, literal })
    }
}
