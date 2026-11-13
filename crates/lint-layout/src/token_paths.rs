use std::iter;

use proc_macro2::{Ident, Punct, TokenStream, TokenTree};
use syn::Path;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseBuffer, ParseStream};

#[derive(Debug)]
pub struct TokenPaths(Vec<Path>);

impl TokenPaths {
    const ACCESS: char = '.';

    #[must_use]
    pub fn within(tokens: TokenStream) -> Vec<Path> {
        syn::parse2::<TokenPaths>(tokens).map_or_default(|paths| paths.0)
    }

    fn attempted(input: ParseStream, attempt: &ParseBuffer) -> syn::Result<Vec<Path>> {
        match attempt.call(Path::parse_mod_style) {
            Ok(path) => Ok(Self::advanced(input, attempt, path)),
            Err(_) => Self::skipped(input.parse()?, input),
        }
    }

    fn advanced(input: ParseStream, attempt: &ParseBuffer, path: Path) -> Vec<Path> {
        input.advance_to(attempt);
        path.segments.first().map(|_| ()).map(|()| path).into_iter().collect()
    }

    fn skipped(tree: TokenTree, input: ParseStream) -> syn::Result<Vec<Path>> {
        match tree {
            TokenTree::Group(group) => Ok(Self::within(group.stream())),
            TokenTree::Punct(punct) => Self::punctuated(&punct, input),
            _ => Ok(Vec::new()),
        }
    }

    fn punctuated(punct: &Punct, input: ParseStream) -> syn::Result<Vec<Path>> {
        Some(punct.as_char())
            .filter(|spelled| *spelled == Self::ACCESS)
            .map_or_else(|| Ok(Vec::new()), |_| input.parse::<Option<Ident>>().map(|_| Vec::new()))
    }
}

impl Parse for TokenPaths {
    fn parse(input: ParseStream) -> syn::Result<TokenPaths> {
        let steps = iter::from_fn(|| input.cursor().token_tree().map(|_| Self::attempted(input, &input.fork())));
        steps
            .collect::<syn::Result<Vec<Vec<Path>>>>()
            .map(|parsed| TokenPaths(parsed.into_iter().flatten().collect()))
    }
}
