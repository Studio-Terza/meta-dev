use proc_macro2::TokenStream;

use lint_layout::{Position, TokenPaths as Parsed};

use crate::namespace::UsePath;

#[derive(Debug)]
pub(super) struct Spelled {
    path: UsePath,
    started_at: Position,
}

impl Spelled {
    pub(super) fn handed<Made>(self, to: impl FnOnce(UsePath, Position) -> Made) -> Made {
        to(self.path, self.started_at)
    }
}

#[derive(Debug)]
pub(super) struct TokenPaths;

impl TokenPaths {
    pub(super) fn within(tokens: TokenStream) -> Vec<Spelled> {
        let parsed = Parsed::within(tokens);
        let spelled = parsed.iter().filter_map(|path| {
            let first = path.segments.first()?;
            Some(Spelled { path: UsePath::from(path), started_at: Position::from(first.ident.span()) })
        });
        spelled.collect()
    }
}
