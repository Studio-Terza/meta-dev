use std::fmt;
use std::ops::Add;

use syn::ItemMod;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use crate::gate::Gate;
use crate::source::{Place, Source};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Budget {
    lines: usize,
    test_lines: usize,
}

#[derive(Debug, Default)]
struct TestScan {
    test_lines: usize,
    gated: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exceeded(Budget);

impl Budget {
    const PERCENT_OF_LINES: usize = 18;
    const WHOLE: usize = 100;

    pub(crate) fn of_file(source: &Source, file: &syn::File, gate: Gate) -> Budget {
        let mut scan = TestScan::default();
        scan.visit_file(file);
        let lines = source.text().bytes().filter(|byte| *byte == b'\n').count();
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped);
        Budget { lines, test_lines: shipped.map_or(lines, |()| scan.test_lines) }
    }

    pub(crate) fn exceeded(self) -> Option<Exceeded> {
        (self.test_lines * Budget::WHOLE > self.lines * Budget::PERCENT_OF_LINES).then_some(Exceeded(self))
    }
}

impl<'ast> Visit<'ast> for TestScan {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Gate::of(&module.attrs) {
            Gate::Entered => visit::visit_item_mod(self, module),
            Gate::Skipped => {
                let span = module.span();
                let outermost = (self.gated == 0).then(|| span.end().line - span.start().line + 1);
                self.test_lines += outermost.unwrap_or_default();
                self.gated += 1;
                visit::visit_item_mod(self, module);
                self.gated -= 1;
            }
        }
    }
}

impl Add for Budget {
    type Output = Budget;

    fn add(self, other: Budget) -> Budget {
        Budget { lines: self.lines + other.lines, test_lines: self.test_lines + other.test_lines }
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
