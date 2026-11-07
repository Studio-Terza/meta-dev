use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{FnArg, Type};

use lint_layout::Name;

#[derive(Debug)]
pub struct Erasure<'found> {
    self_type: &'found Name,
    inner: Option<&'found Type>,
}

impl<'found> Erasure<'found> {
    const OWN_TYPE: &'static str = "Self";
    const INNER: &'static str = "Inner";

    pub fn around(self_type: &'found Name, inner: Option<&'found Type>) -> Self {
        Erasure { self_type, inner }
    }

    pub fn tokens(&self, tokens: TokenStream) -> TokenStream {
        tokens.into_iter().map(|token| self.token(token)).collect()
    }

    pub fn spelled_argument(&self, argument: &FnArg) -> TokenStream {
        match argument {
            FnArg::Receiver(receiver) => self.spelled(receiver),
            FnArg::Typed(typed) => self.spelled_input(&typed.ty),
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
            .filter(|ident| *self.self_type != Name::from(ident))
            .unwrap_or_else(|| Ident::new(Self::OWN_TYPE, span))
    }

    pub fn spelled(&self, ty: &impl ToTokens) -> TokenStream {
        self.tokens(ty.to_token_stream())
    }

    pub fn spelled_input(&self, ty: &Type) -> TokenStream {
        self.inner.filter(|inner| *inner == ty).map_or_else(
            || self.spelled(ty),
            |_| TokenStream::from(TokenTree::Ident(Ident::new(Self::INNER, Span::call_site()))),
        )
    }
}
