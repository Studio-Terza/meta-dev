use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{ItemFn, ItemMod, Macro, PathSegment};

use lint_core::Walk;
use lint_core::{Macros, SourceFile};
use lint_layout::TestMarks;
use lint_sweep::{Descend, Facts, Kind, Watch};
use newtype::Counter;

use super::count::Count;

#[derive(Debug)]
pub struct TestScan<'file> {
    file: &'file SourceFile,
    tests: Count,
    test_lines: Count,
    before: Vec<Count>,
}

impl<'file> TestScan<'file> {
    const PROPERTY_BLOCK: &str = "proptest";

    pub(super) fn new(file: &'file SourceFile) -> TestScan<'file> {
        TestScan { file, tests: Count::default(), test_lines: Count::default(), before: Vec::new() }
    }

    pub(super) fn counted<Out>(self, into: impl FnOnce(Count, Count) -> Out) -> Out {
        into(self.tests, self.test_lines)
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

impl<'ast> Watch<'ast> for TestScan<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemFn, Kind::ItemMod, Kind::Macro]
    }

    fn on_item_fn(&mut self, _: &'ast ItemFn, facts: &Facts<'_, ItemFn>) -> Descend {
        self.tests = self.tests.plus(Count::new(facts.test_mark().into_iter().count()));
        Descend::Into
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, _: &Facts<'_, ItemMod>) -> Descend {
        self.before.push(self.test_lines);
        Descend::Into
    }

    fn off_item_mod(&mut self, module: &'ast ItemMod, facts: &Facts<'_, ItemMod>) {
        let before = self.before.pop().unwrap_or_default();
        self.test_lines = Some(facts.gate())
            .filter(|walk| *walk == Walk::Skipped)
            .map_or(self.test_lines, |_| before.plus(Self::line_count(module.span())));
    }

    fn on_macro(&mut self, invocation: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        self.tests = self.tests.plus(self.tests_in(invocation));
        Descend::Over
    }
}
