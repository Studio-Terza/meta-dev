use syntax::Spanned;
use syntax::{Macro, MacroTree};

use graph::{Building, Domain, Quote, Quoted};
use lint_core::{Macros, SourceFile};
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
        let invoked = footing.resolving().path(footing.scope(self.located.scope()), &self.written, Domain::Macros);
        let kept = Some(invoked).filter(|resolved| resolved.std().is_none());
        let line = self.line;
        kept.map(|found| Quoted::new(found, self.quote)).into_iter().fold(building, |mut built, quoted| {
            built.quoted(footing.file(), line, quoted);
            built
        })
    }
}

impl QuoteRow {
    pub(super) fn found(source: &SourceFile, invoked: &Macro, here: &Seat) -> Vec<QuoteRow> {
        let nested = |inner: &Macro| Macros::nested(source, inner).to_vec();
        MacroTree::preorder(invoked, &nested).iter().filter_map(|found| QuoteRow::row(found, here)).collect()
    }

    fn row(invoked: &Macro, here: &Seat) -> Option<QuoteRow> {
        Quotation::first(invoked).map(|literal| QuoteRow {
            located: here.located(Position::from(invoked.path.span())),
            written: UsePath::from(&invoked.path),
            line: LineNumber::from(literal.span()),
            quote: Quotation::quote(&literal),
        })
    }
}
