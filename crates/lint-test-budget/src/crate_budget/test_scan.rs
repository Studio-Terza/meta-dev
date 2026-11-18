use algebra::{Monoid, Semigroup};
use syntax::Span;
use syntax::Spanned;
use syntax::{ItemFn, ItemMod, Macro, PathSegment};

use lint_core::Walk;
use lint_core::{Macros, SourceFile};
use lint_layout::TestMarks;
use lint_sweep::{Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope};
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

impl Tally {
    pub(super) fn counted<Out>(self, into: impl FnOnce(Count, Count) -> Out) -> Out {
        into(self.tests, self.test_lines)
    }

    fn tested(tests: Count) -> Tally {
        Tally { tests, test_lines: Count::default() }
    }
}

impl Semigroup for Tally {
    fn combine(self, other: Tally) -> Tally {
        Tally { tests: self.tests.plus(other.tests), test_lines: self.test_lines.plus(other.test_lines) }
    }
}

impl Monoid for Tally {}

impl Contribute for TestScan<'_> {
    type Open = ();
    type Part = Tally;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemFn, Kind::Macro]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemMod)] }
    }

    fn on_item_fn(
        &self,
        _: &ItemFn,
        facts: &Facts<'_, ItemFn>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Tally> {
        Contributed::parted(Tally::tested(Count::new(facts.test_mark().into_iter().count())))
    }

    fn off_item_mod(&self, module: &ItemMod, facts: &Facts<'_, ItemMod>, inner: Contribution<(), Tally>) -> Tally {
        let within = inner.concluded(|(), tally| tally);
        Tally {
            test_lines: Some(facts.gate())
                .filter(|walk| *walk == Walk::Skipped)
                .map_or(within.test_lines, |_| Self::line_count(module.span())),
            ..within
        }
    }

    fn on_macro(
        &self,
        invocation: &Macro,
        _: &Facts<'_, Macro>,
        _around: lint_sweep::Around<'_, Self::Open, Self::Part>,
    ) -> Contributed<(), Tally> {
        Contributed::new((), Tally::tested(self.tests_in(invocation)), Descend::Over)
    }
}
