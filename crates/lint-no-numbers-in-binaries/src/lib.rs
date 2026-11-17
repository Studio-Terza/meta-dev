use std::path::Path;

use lint_core::{Headline, Violation};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Finds, Kind};
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
        Box::new(Contributing::new(Numbers { path: AsRef::<Path>::as_ref(file.head()), kinds }))
    }
}

#[derive(Debug)]
struct Numbers<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
}

impl Contribute for Numbers<'_> {
    type Open = ();
    type Part = Vec<LineNumber>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Vec<LineNumber>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_lit_int(
        &self,
        _: &LitInt,
        facts: &Facts<'_, LitInt>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Vec<LineNumber>> {
        Contributed::parted(facts.unnamed().into_iter().collect())
    }

    fn on_lit_float(
        &self,
        _: &LitFloat,
        facts: &Facts<'_, LitFloat>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Vec<LineNumber>> {
        Contributed::parted(facts.unnamed().into_iter().collect())
    }
}

impl<'file> Concludes<'file> for Numbers<'file> {
    fn violations(self, (): (), lines: Vec<LineNumber>) -> Vec<Violation<'file>> {
        lines
            .into_iter()
            .map(|line| Violation::new(Site::new(self.path, line), NoNumbersInBinaries::IN_BINARY))
            .collect()
    }
}
