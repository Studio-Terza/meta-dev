use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Ident, Signature};

#[derive(Debug)]
pub(super) struct Constructor;

impl Constructor {
    const SELF: &str = "Self";

    pub(super) fn made_by<'signature>(
        signature: &'signature Signature,
        owner: &Ident,
    ) -> Option<&'signature Signature> {
        match signature.receiver() {
            Some(_) => None,
            None => Self::mention(signature.output.to_token_stream(), owner).map(|_| signature),
        }
    }

    fn mention(tokens: TokenStream, owner: &Ident) -> Option<Ident> {
        tokens.into_iter().find_map(|tree| Self::mentioned(tree, owner))
    }

    fn mentioned(tree: TokenTree, owner: &Ident) -> Option<Ident> {
        match tree {
            TokenTree::Ident(ident) => {
                (ident == Self::SELF).then_some(()).or((ident == *owner).then_some(())).map(|()| ident)
            }
            TokenTree::Group(group) => Self::mention(group.stream(), owner),
            TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }
}
