use std::path::Path;

use lint_core::{Headline, Violation};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};
use syntax::{ItemMod, LitFloat, LitInt};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct NoNumbersInBinaries;

impl NoNumbersInBinaries {
    const IN_BINARY: Message = Message::new(
        Problem::new("numeric literal in a binary"),
        Fix::new("read it from the settings file or its embedded defaults"),
    );
}

impl SweptLint for NoNumbersInBinaries {
    const HEADLINE: Headline = Headline::new("a binary is wiring: a number it needs is a setting in its config file");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        let kinds = file.head().binary().map_or(&[][..], |_| &[Kind::ItemMod, Kind::LitInt, Kind::LitFloat][..]);
        Box::new(Folding::from(Numbers { path: AsRef::<Path>::as_ref(file.head()), kinds }))
    }
}

#[derive(Debug)]
struct Numbers<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
}

impl<'file> Numbers<'file> {
    fn numbered(&self, found: Vec<Violation<'file>>, unnamed: Option<LineNumber>) -> Visit<Vec<Violation<'file>>> {
        Visit::adding(
            found,
            unnamed.map(|line| Violation::new(Site::new(self.path, line), NoNumbersInBinaries::IN_BINARY)),
            Descend::Into,
        )
    }
}

impl<'file> Fold for Numbers<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&self, found: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_lit_int(&self, found: Self::Found, _: &LitInt, facts: &Facts<'_, LitInt>) -> Visit<Self::Found> {
        self.numbered(found, facts.unnamed())
    }

    fn on_lit_float(&self, found: Self::Found, _: &LitFloat, facts: &Facts<'_, LitFloat>) -> Visit<Self::Found> {
        self.numbered(found, facts.unnamed())
    }
}
