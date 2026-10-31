use proc_macro2::{TokenStream, TokenTree};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Block, Expr, Macro, Stmt, Token};

#[derive(Debug)]
pub struct Macros;

impl Macros {
    #[must_use]
    pub fn statements(invocation: &Macro) -> Vec<Stmt> {
        if invocation.path.is_ident("macro_rules") {
            Self::transcribers(&invocation.tokens).into_iter().flat_map(Self::parsed).collect()
        } else {
            Self::parsed(invocation.tokens.clone())
        }
    }

    fn parsed(tokens: TokenStream) -> Vec<Stmt> {
        Block::parse_within
            .parse2(tokens.clone())
            .or_else(|_| {
                Punctuated::<Expr, Token![,]>::parse_terminated
                    .parse2(tokens)
                    .map(|arguments| arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect())
            })
            .unwrap_or_default()
    }

    fn transcribers(rules: &TokenStream) -> Vec<TokenStream> {
        let mut bodies = Vec::new();
        let mut after_arrow = false;
        for tree in rules.clone() {
            match tree {
                TokenTree::Punct(punct) if punct.as_char() == '>' => after_arrow = true,
                TokenTree::Group(group) if after_arrow => {
                    bodies.push(group.stream());
                    after_arrow = false;
                }
                _ => after_arrow = false,
            }
        }
        bodies
    }
}
