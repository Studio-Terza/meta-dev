use syn::Macro;
use syn::spanned::Spanned;

use graph::{Building, Domain, Quote, Quoted};
use lint_core::MacroCalls;
use lint_layout::{LineNumber, Position};

use crate::namespace::UsePath;

use super::boundary::Quotation;
use super::seat::{Built, Footing, Located, Seat};

#[derive(Debug)]
pub(super) struct QuoteRow {
    located: Located,
    written: UsePath,
    line: LineNumber,
    quote: Quote,
}

impl Built for QuoteRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let invoked = footing.resolving().path(self.located.scope(), &self.written, Domain::Macros);
        let kept = Some(invoked).filter(|resolved| resolved.std().is_none());
        let line = self.line;
        kept.map(|found| Quoted::new(found, self.quote)).into_iter().fold(building, |mut built, quoted| {
            built.quoted(footing.file(), line, quoted);
            built
        })
    }
}

impl QuoteRow {
    pub(super) fn found(invoked: &Macro, here: &Seat) -> Vec<QuoteRow> {
        let row = Quotation::first(invoked).map(|literal| QuoteRow {
            located: here.located(Position::from(invoked.path.span())),
            written: UsePath::from(&invoked.path),
            line: LineNumber::from(literal.span()),
            quote: Quotation::quote(&literal),
        });
        let nested =
            MacroCalls::within(invoked).iter().flat_map(|inner| QuoteRow::found(inner, here)).collect::<Vec<_>>();
        row.into_iter().chain(nested).collect()
    }
}
