use std::path::Path;

use lint_core::{Headline, Violation};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Kind, Watch};
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
        Box::new(Numbers { path: AsRef::<Path>::as_ref(file.head()), kinds, lines: Vec::new() })
    }
}

#[derive(Debug)]
struct Numbers<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
    lines: Vec<LineNumber>,
}

impl<'ast> Watch<'ast> for Numbers<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_lit_int(&mut self, _: &'ast LitInt, facts: &Facts<'_, LitInt>) -> Descend {
        self.lines.extend(facts.unnamed());
        Descend::Into
    }

    fn on_lit_float(&mut self, _: &'ast LitFloat, facts: &Facts<'_, LitFloat>) -> Descend {
        self.lines.extend(facts.unnamed());
        Descend::Into
    }
}

impl<'file> Finds<'file> for Numbers<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.path;
        self.lines
            .into_iter()
            .map(|line| Violation::new(Site::new(path, line), NoNumbersInBinaries::IN_BINARY))
            .collect()
    }
}
