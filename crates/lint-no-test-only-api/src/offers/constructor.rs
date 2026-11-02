use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Ident, Signature};

#[derive(Debug)]
pub(super) struct Constructor;

impl Constructor {
    const SELF: &str = "Self";

    pub(super) fn is_made_by(signature: &Signature, owner: &Ident) -> bool {
        signature.receiver().is_none() && Self::mentions(signature.output.to_token_stream(), owner)
    }

    fn mentions(tokens: TokenStream, owner: &Ident) -> bool {
        tokens.into_iter().any(|tree| match tree {
            TokenTree::Ident(ident) => ident == Self::SELF || ident == *owner,
            TokenTree::Group(group) => Self::mentions(group.stream(), owner),
            TokenTree::Punct(_) | TokenTree::Literal(_) => false,
        })
    }
}
