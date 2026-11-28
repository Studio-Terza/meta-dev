use std::fmt;
use std::sync::Arc;

use pipeline::{Limit, Verdict};
use syn::spanned::Spanned;
use syn::{ItemMod, Macro};

use crate::finding::Finding;
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Fact, Lint, Reads};
use crate::source::{Place, Source};
use crate::workspace::Package;

pipeline::measured! {
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Budget {
        lines: usize,
        test_lines: usize,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TestBudget {
    test_lines: usize,
    gated: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exceeded(Budget);

impl Budget {
    const PERCENT_OF_LINES: usize = 18;
    const WHOLE: usize = 100;

    pub(crate) fn exceeded(self) -> Option<Exceeded> {
        let limit = Limit::at(self.lines * Budget::PERCENT_OF_LINES);
        (limit.judged(&(self.test_lines * Budget::WHOLE)) == Verdict::Over).then_some(Exceeded(self))
    }
}

impl Fact for Budget {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        Vec::new()
    }
}

impl Hooks for TestBudget {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let skipped = Gate::of(&module.attrs).entered().xor(Some(()));
        skipped.map_or_default(|()| {
            let span = module.span();
            let outermost = (self.gated == 0).then(|| span.end().line - span.start().line + 1);
            self.test_lines += outermost.unwrap_or_default();
            self.gated += 1;
        });
        Descent::Into
    }

    fn left_item_mod(&mut self, module: &ItemMod) {
        self.gated -= usize::from(Gate::of(&module.attrs) == Gate::Skipped);
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Nothing
    }
}

impl<'file> Lint<'file> for TestBudget {
    const HEADLINE: &'static str = "test budget";
    const READS: Reads = Reads::All;

    type Fact = Budget;

    fn new(_: &'file Source) -> TestBudget {
        TestBudget::default()
    }

    fn fact(self, source: &'file Source) -> Budget {
        let lines = source.text().bytes().filter(|byte| *byte == b'\n').count();
        let shipped = source.gate().entered().filter(|()| source.place() == Place::Shipped);
        Budget { lines, test_lines: shipped.map_or(lines, |()| self.test_lines) }
    }
}

impl fmt::Display for Budget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "test budget: {} of {} lines are tests, at most {}% may be",
            self.test_lines,
            self.lines,
            Budget::PERCENT_OF_LINES
        )
    }
}

impl fmt::Display for Exceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; cut test lines or test through fewer, table-driven cases", self.0)
    }
}
