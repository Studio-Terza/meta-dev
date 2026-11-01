use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Block, Expr, Macro, Stmt, Token};

#[derive(Debug)]
pub struct Macros;

impl Macros {
    const ARROW_END: char = '>';
    const ATTRIBUTE_MARK: char = '#';

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

    #[must_use]
    pub fn attributes(invocation: &Macro) -> Vec<Attribute> {
        Self::attributes_in(invocation.tokens.clone())
    }

    fn attributes_in(tokens: TokenStream) -> Vec<Attribute> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(1)).filter_map(|(mark, body)| match (mark, body) {
            (TokenTree::Punct(punct), TokenTree::Group(group))
                if punct.as_char() == Self::ATTRIBUTE_MARK && group.delimiter() == Delimiter::Bracket =>
            {
                Attribute::parse_outer.parse2(quote!(#mark #body)).ok()
            }
            _ => None,
        });
        let nested = trees.iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(group.stream()),
            _ => None,
        });
        here.flatten().chain(nested.flat_map(Self::attributes_in)).collect()
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
