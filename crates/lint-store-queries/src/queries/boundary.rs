use proc_macro2::TokenTree;
use sqlformat::{FormatOptions, QueryParams};
use syn::{LitStr, Macro};

use lint_layout::{LineNumber, UsePath, Word, words};

#[derive(Debug)]
pub(super) struct Sql(LitStr);

impl Sql {
    const PATH: [Word<'static>; 2] = words!["store_ports", "query"];

    pub(super) fn queried(invocation: &Macro, candidates: &[UsePath]) -> Option<Sql> {
        candidates
            .iter()
            .find_map(|candidate| candidate.spells(&Self::PATH))
            .map(|_| invocation)
            .and_then(|query| query.tokens.clone().into_iter().find_map(Self::literal))
            .map(Sql)
    }

    pub(super) fn unformatted(&self) -> Option<LineNumber> {
        let written = self.0.token().to_string();
        let formatted = sqlformat::format(&self.0.value(), &QueryParams::None, &FormatOptions::default());
        (written != format!("r#\"\n{formatted}\n\"#")).then_some(LineNumber::from(self.0.span()))
    }

    fn literal(tree: TokenTree) -> Option<LitStr> {
        match tree {
            TokenTree::Literal(literal) => syn::parse2(TokenTree::Literal(literal).into()).ok(),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Punct(_) => None,
        }
    }
}
