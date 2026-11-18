mod ignored;

use sqlformat::{FormatOptions, QueryParams};
use syntax::LitStr;

use graph::Quote;

use super::unformatted::Unformatted;
use crate::queries::boundary::ignored::Ignored;
use invariants::Ignoring;

#[derive(Debug)]
pub(super) struct Sql<'model>(&'model Quote);

impl<'model> From<&'model Quote> for Sql<'model> {
    fn from(quote: &'model Quote) -> Sql<'model> {
        Sql(quote)
    }
}

impl Sql<'_> {
    pub(super) fn unquoted(&self) -> Option<String> {
        syntax::parse::parse_str::<LitStr>(self.0.as_ref()).ignoring(Ignored::Unparsed).map(|literal| literal.value())
    }

    pub(super) fn unformatted(&self) -> Option<Unformatted> {
        let written: &str = self.0.as_ref();
        let formatted = sqlformat::format(&self.unquoted()?, &QueryParams::None, &FormatOptions::default());
        (written != format!("r#\"\n{formatted}\n\"#")).then_some(Unformatted)
    }
}
