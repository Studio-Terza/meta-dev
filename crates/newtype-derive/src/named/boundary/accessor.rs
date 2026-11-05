use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Ident, Lit, Token, Type};

use super::VariantName;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Literal {
    Text,
    Integer,
    Boolean,
    Character,
}

impl Literal {
    const SPELLINGS: [(&'static str, Literal); 15] = [
        ("&'static str", Literal::Text),
        ("bool", Literal::Boolean),
        ("char", Literal::Character),
        ("u8", Literal::Integer),
        ("u16", Literal::Integer),
        ("u32", Literal::Integer),
        ("u64", Literal::Integer),
        ("u128", Literal::Integer),
        ("usize", Literal::Integer),
        ("i8", Literal::Integer),
        ("i16", Literal::Integer),
        ("i32", Literal::Integer),
        ("i64", Literal::Integer),
        ("i128", Literal::Integer),
        ("isize", Literal::Integer),
    ];
}

impl TryFrom<&Type> for Literal {
    type Error = Error;

    fn try_from(ty: &Type) -> syn::Result<Literal> {
        Self::SPELLINGS
            .iter()
            .find(|(spelling, _)| syn::parse_str::<Type>(spelling).is_ok_and(|parsed| parsed == *ty))
            .map(|(_, literal)| *literal)
            .ok_or_else(|| Error::new_spanned(ty, Accessor::UNSUPPORTED))
    }
}

impl TryFrom<&Lit> for Literal {
    type Error = Error;

    fn try_from(lit: &Lit) -> syn::Result<Literal> {
        match lit {
            Lit::Str(_) => Ok(Literal::Text),
            Lit::Int(_) => Ok(Literal::Integer),
            Lit::Bool(_) => Ok(Literal::Boolean),
            Lit::Char(_) => Ok(Literal::Character),
            _ => Err(Error::new_spanned(lit, Accessor::WRONG_TYPE)),
        }
    }
}

pub struct Accessor {
    name: Ident,
    ty: Type,
    literal: Literal,
}

impl Accessor {
    const UNSUPPORTED: &'static str = "an accessor type is &'static str, an integer type, bool or char";
    const WRONG_TYPE: &'static str = "this literal does not have the accessor's declared type";
    const ATTRIBUTE: &'static str = "accessor";
    const RESERVED: &'static str = "an accessor cannot be called name, which is the variant's own name";
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
        (self.name == "name")
            .then_some(Self::RESERVED)
            .or_else(|| earlier.iter().any(|accessor| accessor.name == self.name).then_some(Self::REPEATED))
            .map(|message| Error::new_spanned(&self.name, message))
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn checked<'lit>(&self, lit: &'lit Lit) -> syn::Result<&'lit Lit> {
        Literal::try_from(lit)
            .and_then(|found| {
                Some(found)
                    .filter(|found| *found == self.literal)
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
        Ok(Accessor { name, ty, literal })
    }
}
