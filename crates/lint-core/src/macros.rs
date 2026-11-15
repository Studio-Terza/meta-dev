use std::rc::Rc;

use syntax::Punctuated;
use syntax::parse::Speculative;
use syntax::parse::{ParseStream, Parser};
use syntax::quote;
use syntax::{Attribute, Block, Expr, Item, Macro, Stmt, Token};
use syntax::{Group, TokenStream, TokenTree};

use crate::source_file::SourceFile;
use crate::syntax::Syntax;
use crate::watch::Address;

mod boundary;

pub use boundary::Expansions;

#[derive(Debug)]
pub struct Macros;

impl Macros {
    const ARROW_END: char = '>';

    #[must_use]
    pub fn items(invocation: &Macro) -> Vec<Item> {
        syntax::parse::parse2::<syntax::File>(invocation.tokens.clone()).map_or_else(|_| Vec::new(), |file| file.items)
    }

    #[must_use]
    pub fn statements(invocation: &Macro) -> Vec<Stmt> {
        let bodies = invocation
            .path
            .get_ident()
            .filter(|ident| *ident == "macro_rules")
            .map_or_else(|| vec![invocation.tokens.clone()], |_| Self::transcribers(invocation.tokens.clone()));
        bodies
            .into_iter()
            .flat_map(|tokens| Self::statements_or_expressions.parse2(tokens).unwrap_or_default())
            .collect()
    }

    #[must_use]
    pub fn expanded(file: &SourceFile, invocation: &Macro) -> Rc<[Stmt]> {
        file.expansions().statements(invocation)
    }

    #[must_use]
    pub fn nested(file: &SourceFile, invocation: &Macro) -> Rc<[Macro]> {
        file.expansions().calls(Address::from(invocation), invocation)
    }

    #[must_use]
    pub fn nested_at(file: &SourceFile, at: Address, copy: &Macro) -> Rc<[Macro]> {
        file.expansions().calls(at, copy)
    }

    #[must_use]
    pub fn items_in_file(file: &SourceFile, invocation: &Macro) -> Rc<[Item]> {
        file.expansions().items(invocation)
    }

    #[must_use]
    pub fn attributes_in_file(file: &SourceFile, invocation: &Macro) -> Rc<[Attribute]> {
        file.expansions().attributes(invocation)
    }

    pub fn pin(file: &SourceFile, invocation: &Macro) {
        file.expansions().pin(invocation);
    }

    pub fn walk(file: &SourceFile, invocation: &Macro, visit: impl FnMut(&Stmt)) {
        Self::expanded(file, invocation).iter().for_each(visit);
    }

    #[must_use]
    pub fn attributes(invocation: &Macro) -> Vec<Attribute> {
        Self::attributes_in(invocation.tokens.clone())
    }

    fn attributes_in(tokens: TokenStream) -> Vec<Attribute> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(1)).filter_map(|(mark, body)| {
            Syntax::punct(mark)
                .and(Syntax::group(body))
                .and_then(|_| Attribute::parse_outer.parse2(quote!(#mark #body)).ok())
        });
        let nested = trees.iter().filter_map(Syntax::group).map(Group::stream);
        here.flatten().chain(nested.flat_map(Self::attributes_in)).collect()
    }

    fn statements_or_expressions(input: ParseStream) -> syntax::Result<Vec<Stmt>> {
        let attempt = input.fork();
        Block::parse_within(&attempt).map_or_else(
            |_| Punctuated::<Expr, Token![,]>::parse_terminated(input).map(Self::expressions),
            |statements| {
                input.advance_to(&attempt);
                Ok(statements)
            },
        )
    }

    fn expressions(arguments: Punctuated<Expr, Token![,]>) -> Vec<Stmt> {
        arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect()
    }

    fn transcribers(rules: TokenStream) -> Vec<TokenStream> {
        let trees: Vec<TokenTree> = rules.into_iter().collect();
        trees
            .iter()
            .zip(trees.iter().skip(1))
            .filter_map(|(arrow, body)| {
                Syntax::punct(arrow).filter(|punct| punct.as_char() == Self::ARROW_END).and(Syntax::group(body))
            })
            .map(Group::stream)
            .collect()
    }
}
