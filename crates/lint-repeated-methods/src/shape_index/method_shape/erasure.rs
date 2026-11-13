use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use syn::FnArg;

use lint_layout::Name;

#[derive(Debug)]
pub struct Erasure<'found> {
    self_type: &'found Name,
}

impl<'found> Erasure<'found> {
    const OWN_TYPE: &'static str = "Self";

    pub fn around(self_type: &'found Name) -> Erasure<'found> {
        Erasure { self_type }
    }

    pub fn tokens(&self, tokens: TokenStream) -> TokenStream {
        tokens.into_iter().map(|token| self.token(token)).collect()
    }

    pub fn spelled_argument(&self, argument: &FnArg) -> TokenStream {
        match argument {
            FnArg::Receiver(receiver) => self.spelled(receiver),
            FnArg::Typed(typed) => self.spelled(&typed.ty),
        }
    }

    fn token(&self, token: TokenTree) -> TokenTree {
        match token {
            TokenTree::Ident(ident) => TokenTree::Ident(self.renamed(ident)),
            TokenTree::Group(group) => TokenTree::Group(Group::new(group.delimiter(), self.tokens(group.stream()))),
            TokenTree::Punct(_) | TokenTree::Literal(_) => token,
        }
    }

    fn renamed(&self, ident: Ident) -> Ident {
        let span = ident.span();
        Some(ident)
            .filter(|candidate| *candidate != *self.self_type)
            .unwrap_or_else(|| Ident::new(Self::OWN_TYPE, span))
    }

    pub fn spelled(&self, ty: &impl ToTokens) -> TokenStream {
        self.tokens(ty.to_token_stream())
    }
}
