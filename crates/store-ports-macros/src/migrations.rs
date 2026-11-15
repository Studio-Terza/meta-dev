mod boundary;

use syntax::LitStr;
use syntax::parse::{Parse, ParseStream};
use syntax::quote;
use syntax::{Span, TokenStream};

use crate::expanded::Expanding;
use boundary::Directory;

#[derive(Debug)]
pub struct Migrations(Option<LitStr>);

impl Parse for Migrations {
    fn parse(input: ParseStream<'_>) -> syntax::Result<Migrations> {
        input.parse().map(Migrations)
    }
}

impl Expanding for Migrations {
    fn expansion(self) -> syntax::Result<TokenStream> {
        let span = self.0.as_ref().map_or_else(Span::call_site, LitStr::span);
        let found =
            Directory::named(self.0.as_ref()).migrations().map_err(|failed| syntax::Error::new(span, failed))?;
        let from = quote!(::store_ports::Migrator::from);
        Ok(quote!(#from(::std::vec![#(#found),*])))
    }
}
