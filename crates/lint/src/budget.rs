use std::fmt;
use std::ops::Add;

use syn::spanned::Spanned;
use syn::{ItemMod, Macro};

use crate::gate::Gate;
use crate::source::{Place, Source};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Budget {
    lines: usize,
    test_lines: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TestScan {
    test_lines: usize,
    gated: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exceeded(Budget);

impl Budget {
    const PERCENT_OF_LINES: usize = 18;
    const WHOLE: usize = 100;

    pub(crate) fn of_file(source: &Source, scan: TestScan, gate: Gate) -> Budget {
        let lines = source.text().bytes().filter(|byte| *byte == b'\n').count();
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped);
        Budget { lines, test_lines: shipped.map_or(lines, |()| scan.test_lines) }
    }

    pub(crate) fn exceeded(self) -> Option<Exceeded> {
        (self.test_lines * Budget::WHOLE > self.lines * Budget::PERCENT_OF_LINES).then_some(Exceeded(self))
    }
}

impl Hooks for TestScan {
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
