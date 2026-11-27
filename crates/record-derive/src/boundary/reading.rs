use proc_macro2::TokenStream;
use quote::quote;
use strum::{IntoStaticStr, VariantArray};
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, Expr, Field, Ident, Meta, Token, Type};

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Keyword {
    Field,
    Optional,
    Repeated,
    Record,
}

#[derive(Debug)]
enum Taking {
    Field(Type),
    Optional(Type),
    Repeated(Type),
    Record,
}

#[derive(Debug)]
struct Spec {
    taking: Taking,
    via: Option<Expr>,
}

#[derive(Debug)]
pub(crate) struct Reading {
    ident: Ident,
    ty: Type,
    spec: Spec,
    past: Vec<Expr>,
}

impl Keyword {
    fn named(attribute: &Attribute) -> Option<Keyword> {
        Self::VARIANTS.iter().copied().find(|keyword| attribute.path().is_ident(<&str>::from(*keyword)))
    }

    fn spec(self, attribute: &Attribute) -> syn::Result<Spec> {
        match self {
            Keyword::Field => attribute.parse_args_with(|input: ParseStream| Spec::typed(input, Taking::Field)),
            Keyword::Optional => attribute.parse_args_with(|input: ParseStream| Spec::typed(input, Taking::Optional)),
            Keyword::Repeated => attribute.parse_args_with(|input: ParseStream| Spec::typed(input, Taking::Repeated)),
            Keyword::Record => Spec::nested(&attribute.meta),
        }
    }
}

impl Spec {
    fn typed(input: ParseStream, taking: fn(Type) -> Taking) -> syn::Result<Spec> {
        let ty: Type = input.parse()?;
        Ok(Spec { taking: taking(ty), via: Self::via(input)? })
    }

    fn nested(meta: &Meta) -> syn::Result<Spec> {
        match meta {
            Meta::Path(_) => Ok(Spec { taking: Taking::Record, via: None }),
            Meta::List(_) | Meta::NameValue(_) => Err(Error::new_spanned(meta, "a nested record is a bare #[record]")),
        }
    }

    fn via(input: ParseStream) -> syn::Result<Option<Expr>> {
        let comma = input.parse::<Option<Token![,]>>()?;
        let key = comma.map(|_| input.parse::<Ident>()).transpose()?;
        let keyed = key
            .filter(|named| named != "via")
            .map_or(Ok(()), |named| Err(Error::new_spanned(named, "the only option is via = path")));
        keyed?;
        comma.map(|_| input.parse::<Token![=]>().and_then(|_| input.parse())).transpose()
    }
}

impl Reading {
    pub(crate) fn declared(field: &Field) -> syn::Result<Reading> {
        let ident = field.ident.clone().ok_or_else(|| Error::new_spanned(field, "a record's fields are named"))?;
        let found =
            field.attrs.iter().find_map(|attribute| Keyword::named(attribute).map(|keyword| (keyword, attribute)));
        let (keyword, attribute) = found.ok_or_else(|| {
            Error::new_spanned(field, "mark each field #[field(Kind)], #[optional(Kind)] or #[record]")
        })?;
        let spec = keyword.spec(attribute)?;
        let lists = field.attrs.iter().filter(|listed| listed.path().is_ident("past"));
        let past = lists
            .map(|listed| listed.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated))
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(Reading { ident, ty: field.ty.clone(), spec, past: past.into_iter().flatten().collect() })
    }

    pub(crate) fn ident(&self) -> &Ident {
        &self.ident
    }

    pub(crate) fn field_type(&self) -> &Type {
        &self.ty
    }

    pub(crate) fn stepped<'earlier>(&self, into: &Ident, earlier: impl Iterator<Item = &'earlier Ident>) -> TokenStream {
        let past = &self.past;
        let rest = past.first().map_or_else(|| quote!(self.__fields), |_| quote!(self.__fields.past(&[#(#past),*])?));
        let read = self.spec.taking.read(&rest, &self.ty);
        let converted = self.spec.via.as_ref().map_or_else(|| quote!(__value), |via| self.spec.taking.via(via));
        let ident = &self.ident;
        let kept: Vec<&Ident> = earlier.collect();
        quote! {
            ::core::result::Result::Ok(#read.concluded(|__value, __fields| #into {
                #(#kept: self.#kept,)*
                #ident: #converted,
                __fields,
            }))
        }
    }
}

impl Taking {
    fn read(&self, rest: &TokenStream, ty: &Type) -> TokenStream {
        match self {
            Taking::Field(kind) => quote!(#rest.field::<#kind>()?),
            Taking::Optional(kind) => quote!(#rest.optional::<#kind>()),
            Taking::Repeated(kind) => quote!(#rest.repeated::<#kind>()),
            Taking::Record => quote!(#rest.record::<#ty>()?),
        }
    }

    fn via(&self, via: &Expr) -> TokenStream {
        match self {
            Taking::Field(_) | Taking::Record => quote!((#via)(__value)),
            Taking::Optional(_) => quote!(__value.map(#via)),
            Taking::Repeated(_) => quote!(::core::iter::Iterator::collect(::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(__value),
                #via
            ))),
        }
    }
}
