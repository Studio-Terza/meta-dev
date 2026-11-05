use proc_macro2::{TokenStream, TokenTree};
use syn::Macro;
use syn::parse::Parse;

pub struct Declaring;

impl Declaring {
    const FUNCTION: &'static str = "fn";

    pub fn within(invocation: &Macro) -> Option<&Macro> {
        invocation
            .parse_body_with(TokenStream::parse)
            .into_iter()
            .flat_map(TokenStream::into_iter)
            .flat_map(Self::flattened)
            .find(|tree| matches!(tree, TokenTree::Ident(ident) if ident == Self::FUNCTION))
            .map(|_| invocation)
    }

    fn flattened(tree: TokenTree) -> Vec<TokenTree> {
        match tree {
            TokenTree::Group(group) => group.stream().into_iter().flat_map(Self::flattened).collect(),
            other => vec![other],
        }
    }
}
