use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Error, FnArg, Ident, Pat, PatIdent, PatType, Type};

use super::returned::Returned;

pub struct Parameter {
    name: Ident,
    tree: Type,
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let tree = &self.tree;
        tokens.extend(quote! { #name: #tree });
    }
}

impl Parameter {
    const PLAIN: &'static str = "port method parameters must be plain identifiers";

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn parsed(argument: &FnArg) -> syn::Result<Parameter> {
        let typed = Self::typed(argument)?;
        let name = Self::identifier(&typed.pat)?;
        Returned::refuse_held_parameter(&typed.ty)?;
        Ok(Parameter { name, tree: (*typed.ty).clone() })
    }

    fn typed(argument: &FnArg) -> syn::Result<&PatType> {
        match argument {
            FnArg::Typed(typed) => Ok(typed),
            FnArg::Receiver(receiver) => Err(Error::new_spanned(receiver, "a receiver may only come first")),
        }
    }

    fn identifier(pattern: &Pat) -> syn::Result<Ident> {
        match pattern {
            Pat::Ident(plain) => Self::unadorned(plain),
            other => Err(Error::new_spanned(other, Self::PLAIN)),
        }
    }

    fn unadorned(plain: &PatIdent) -> syn::Result<Ident> {
        plain
            .by_ref
            .is_none()
            .then_some(&plain.ident)
            .filter(|_| plain.subpat.is_none())
            .cloned()
            .ok_or_else(|| Error::new_spanned(plain, Self::PLAIN))
    }
}
