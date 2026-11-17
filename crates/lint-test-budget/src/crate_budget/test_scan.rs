use syntax::Span;
use syntax::Spanned;
use syntax::{ItemFn, ItemMod, Macro, PathSegment};

use lint_core::Extended;
use lint_core::Walk;
use lint_core::{Macros, SourceFile};
use lint_layout::TestMarks;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};
use newtype::Counter;

use super::count::Count;

#[derive(Debug)]
pub struct TestScan<'file> {
    file: &'file SourceFile,
}

#[derive(Debug, Default)]
pub struct Tally {
    tests: Count,
    test_lines: Count,
    before: Vec<Count>,
}

impl Tally {
    pub(super) fn counted<Out>(self, into: impl FnOnce(Count, Count) -> Out) -> Out {
        into(self.tests, self.test_lines)
    }

    fn tested(self, more: Count) -> Tally {
        Tally { tests: self.tests.plus(more), ..self }
    }
}

impl<'file> TestScan<'file> {
    const PROPERTY_BLOCK: &str = "proptest";

    pub(super) fn new(file: &'file SourceFile) -> TestScan<'file> {
        TestScan { file }
    }

    fn line_count(span: Span) -> Count {
        Count::new(span.end().line - span.start().line + 1)
    }

    fn property_block(invocation: &Macro) -> Option<&PathSegment> {
        invocation.path.segments.last().filter(|segment| segment.ident == Self::PROPERTY_BLOCK)
    }

    fn tests_in(&self, invocation: &Macro) -> Count {
        TestMarks::law(invocation)
            .map(|_| Count::new(1))
            .or_else(|| {
                Self::property_block(invocation).map(|_| {
                    Count::new(
                        Macros::attributes_in_file(self.file, invocation).iter().filter_map(TestMarks::test).count(),
                    )
                })
            })
            .unwrap_or_default()
    }
}

impl Fold for TestScan<'_> {
    type Found = Tally;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemFn, Kind::ItemMod, Kind::Macro]
    }

    fn on_item_fn(&self, found: Tally, _: &ItemFn, facts: &Facts<'_, ItemFn>) -> Visit<Tally> {
        Visit::descending(found.tested(Count::new(facts.test_mark().into_iter().count())))
    }

    fn on_item_mod(&self, found: Tally, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Visit<Tally> {
        let before = found.before.extended([found.test_lines]);
        Visit::descending(Tally { before, ..found })
    }

    fn off_item_mod(&self, found: Tally, module: &ItemMod, facts: &Facts<'_, ItemMod>) -> Tally {
        let mut outer = found.before;
        let before = outer.pop().unwrap_or_default();
        let test_lines = Some(facts.gate())
            .filter(|walk| *walk == Walk::Skipped)
            .map_or(found.test_lines, |_| before.plus(Self::line_count(module.span())));
        Tally { test_lines, before: outer, ..found }
    }

    fn on_macro(&self, found: Tally, invocation: &Macro, _: &Facts<'_, Macro>) -> Visit<Tally> {
        Visit::new(found.tested(self.tests_in(invocation)), Descend::Over)
    }
}
