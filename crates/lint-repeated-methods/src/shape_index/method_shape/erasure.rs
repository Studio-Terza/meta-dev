use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Type;

use lint_layout::Name;

#[derive(Debug)]
pub struct Erasure<'found> {
    self_type: &'found Name,
    inner: Option<&'found Type>,
}

impl<'found> Erasure<'found> {
    const OWN_TYPE: &'static str = "Self";
    const INNER: &'static str = "Inner";

    pub fn around(self_type: &'found Name, inner: Option<&'found Type>) -> Erasure<'found> {
        Erasure { self_type, inner }
    }

    pub fn tokens(&self, tokens: TokenStream) -> TokenStream {
        tokens
            .into_iter()
            .map(|token| match token {
                TokenTree::Ident(ident) if *self.self_type == ident => {
                    TokenTree::Ident(Ident::new(Self::OWN_TYPE, ident.span()))
                }
                TokenTree::Group(group) => TokenTree::Group(Group::new(group.delimiter(), self.tokens(group.stream()))),
                TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => token,
            })
            .collect()
    }

    pub fn spelled(&self, ty: &impl ToTokens) -> TokenStream {
        self.tokens(ty.to_token_stream())
    }

    pub fn spelled_input(&self, ty: &Type) -> TokenStream {
        if self.inner == Some(ty) {
            TokenStream::from(TokenTree::Ident(Ident::new(Self::INNER, Span::call_site())))
        } else {
            self.spelled(ty)
        }
    }
}
