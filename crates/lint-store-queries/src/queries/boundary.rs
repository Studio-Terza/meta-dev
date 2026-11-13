use sqlformat::{FormatOptions, QueryParams};
use syn::LitStr;

use graph::Quote;

use super::unformatted::Unformatted;

#[derive(Debug)]
pub(super) struct Sql<'model>(&'model Quote);

impl<'model> From<&'model Quote> for Sql<'model> {
    fn from(quote: &'model Quote) -> Sql<'model> {
        Sql(quote)
    }
}

impl Sql<'_> {
    pub(super) fn unformatted(&self) -> Option<Unformatted> {
        let written: &str = self.0.as_ref();
        let literal = syn::parse_str::<LitStr>(written).ok()?;
        let formatted = sqlformat::format(&literal.value(), &QueryParams::None, &FormatOptions::default());
        (written != format!("r#\"\n{formatted}\n\"#")).then_some(Unformatted)
    }
}
