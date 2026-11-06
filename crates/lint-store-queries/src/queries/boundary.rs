use proc_macro2::TokenTree;
use sqlformat::{FormatOptions, QueryParams};
use syn::{LitStr, Macro};

use lint_layout::LineNumber;

#[derive(Debug)]
pub(super) struct Sql(LitStr);

impl Sql {
    const PATH: [&'static str; 2] = ["store_ports", "query"];

    pub(super) fn queried(invocation: &Macro) -> Option<Sql> {
        let path: Vec<String> = invocation.path.segments.iter().map(|segment| segment.ident.to_string()).collect();
        Some(invocation)
            .filter(|_| path == Self::PATH)
            .and_then(|query| query.tokens.clone().into_iter().find_map(Self::literal))
            .map(Sql)
    }

    pub(super) fn unformatted(&self) -> Option<LineNumber> {
        let written = self.0.token().to_string();
        let formatted = sqlformat::format(self.0.value().trim(), &QueryParams::None, &FormatOptions::default());
        Some(LineNumber::from(self.0.span())).filter(|_| written != format!("r#\"\n{formatted}\n\"#"))
    }

    fn literal(tree: TokenTree) -> Option<LitStr> {
        match tree {
            TokenTree::Literal(literal) => syn::parse2(TokenTree::Literal(literal).into()).ok(),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Punct(_) => None,
        }
    }
}
