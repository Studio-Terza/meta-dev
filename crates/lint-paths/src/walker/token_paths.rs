mod spelled;

use proc_macro2::TokenStream;

use lint_layout::{LineNumber, TokenPaths as Parsed, UsePath};
pub(crate) use spelled::Spelled;

#[derive(Debug)]
pub(crate) struct TokenPaths;

impl TokenPaths {
    pub(crate) fn within(tokens: TokenStream) -> Vec<Spelled> {
        let parsed = Parsed::within(tokens);
        let spelled = parsed.iter().filter_map(|path| {
            let first = path.segments.first()?;
            Some(Spelled::new(UsePath::from(path), LineNumber::from(first.ident.span())))
        });
        spelled.collect()
    }
}
