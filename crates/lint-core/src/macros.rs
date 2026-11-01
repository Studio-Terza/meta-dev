use proc_macro2::{TokenStream, TokenTree};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Block, Expr, Macro, Stmt, Token};

#[derive(Debug)]
pub struct Macros;

impl Macros {
    const ARROW_END: char = '>';

    #[must_use]
    pub fn statements(invocation: &Macro) -> Vec<Stmt> {
        if invocation.path.is_ident("macro_rules") {
            Self::transcribers(&invocation.tokens).into_iter().flat_map(Self::parsed).collect()
        } else {
            Self::parsed(invocation.tokens.clone())
        }
    }

    pub fn walk(invocation: &Macro, visit: impl FnMut(&Stmt)) {
        Self::statements(invocation).iter().for_each(visit);
    }

    fn parsed(tokens: TokenStream) -> Vec<Stmt> {
        Block::parse_within.parse2(tokens.clone()).or_else(|_| Self::expressions(tokens)).unwrap_or_default()
    }

    fn expressions(tokens: TokenStream) -> syn::Result<Vec<Stmt>> {
        Punctuated::<Expr, Token![,]>::parse_terminated
            .parse2(tokens)
            .map(|arguments| arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect())
    }

    fn transcribers(rules: &TokenStream) -> Vec<TokenStream> {
        let trees: Vec<TokenTree> = rules.clone().into_iter().collect();
        trees
            .iter()
            .zip(trees.iter().skip(1))
            .filter_map(|(arrow, body)| match (arrow, body) {
                (TokenTree::Punct(punct), TokenTree::Group(group)) if punct.as_char() == Self::ARROW_END => {
                    Some(group.stream())
                }
                _ => None,
            })
            .collect()
    }
}
