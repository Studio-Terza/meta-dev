use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Macro, Stmt, Token};

#[derive(Debug)]
pub struct Macros;

impl Macros {
    const ARROW_END: char = '>';

    #[must_use]
    pub fn statements(invocation: &Macro) -> Vec<Stmt> {
        let tokens = invocation.tokens.clone();
        let parsed = |tokens| Self::statements_or_expressions.parse2(tokens).unwrap_or_default();
        if invocation.path.is_ident("macro_rules") {
            Self::transcribers(tokens).into_iter().flat_map(parsed).collect()
        } else {
            parsed(tokens)
        }
    }

    pub fn walk(invocation: &Macro, visit: impl FnMut(&Stmt)) {
        Self::statements(invocation).iter().for_each(visit);
    }

    #[must_use]
    pub fn attributes(invocation: &Macro) -> Vec<Attribute> {
        Self::attributes_in(invocation.tokens.clone())
    }

    fn attributes_in(tokens: TokenStream) -> Vec<Attribute> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(1)).filter_map(|(mark, body)| match (mark, body) {
            (TokenTree::Punct(_), TokenTree::Group(_)) => Attribute::parse_outer.parse2(quote!(#mark #body)).ok(),
            _ => None,
        });
        let nested = trees.iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(group.stream()),
            _ => None,
        });
        here.flatten().chain(nested.flat_map(Self::attributes_in)).collect()
    }

    fn statements_or_expressions(input: ParseStream) -> syn::Result<Vec<Stmt>> {
        let attempt = input.fork();
        match Block::parse_within(&attempt) {
            Ok(statements) => {
                input.advance_to(&attempt);
                Ok(statements)
            }
            Err(_) => Punctuated::<Expr, Token![,]>::parse_terminated(input).map(Self::expressions),
        }
    }

    fn expressions(arguments: Punctuated<Expr, Token![,]>) -> Vec<Stmt> {
        arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect()
    }

    fn transcribers(rules: TokenStream) -> Vec<TokenStream> {
        let trees: Vec<TokenTree> = rules.into_iter().collect();
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
