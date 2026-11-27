use std::fmt;
use std::ops::Add;

use syn::spanned::Spanned;
use syn::{ItemMod, Macro};

use crate::gate::Gate;
use crate::source::{Place, Source};
use crate::terms::{Count, Depth};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Budget {
    lines: Count,
    test_lines: Count,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TestScan {
    test_lines: Count,
    gated: Depth,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exceeded(Budget);

impl Budget {
    const PERCENT_OF_LINES: usize = 18;

    pub(crate) fn of_file(source: &Source, scan: TestScan, gate: Gate) -> Budget {
        let lines = source.text().lines();
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped).and(source.package().shipped());
        Budget { lines, test_lines: shipped.map_or(lines, |()| scan.test_lines) }
    }

    pub(crate) fn exceeded(self) -> Option<Exceeded> {
        self.test_lines.over_share(self.lines, Budget::PERCENT_OF_LINES).map(|()| Exceeded(self))
    }
}

impl Hooks for TestScan {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        let skipped = Gate::of(&module.attrs).entered().xor(Some(()));
        let outermost = skipped.and(self.gated.surface()).map(|()| Count::spanning(module.span()));
        self.test_lines = self.test_lines.plus(outermost.unwrap_or_default());
        self.gated = skipped.map_or(self.gated, |()| self.gated.deeper());
        Descent::Into
    }

    fn left_item_mod(&mut self, module: &ItemMod) {
        let skipped = Gate::of(&module.attrs).entered().xor(Some(()));
        self.gated = skipped.map_or(self.gated, |()| self.gated.shallower());
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Nothing
    }
}

impl Add for Budget {
    type Output = Budget;

    fn add(self, other: Budget) -> Budget {
        Budget { lines: self.lines.plus(other.lines), test_lines: self.test_lines.plus(other.test_lines) }
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
