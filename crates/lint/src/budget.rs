mod boundary;

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};
use std::ops::Add;

use newtype::Counter;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use crate::gate::{Gate, TestMark};
use crate::lints::ProcessStarters;
use crate::name::Name;
use crate::source::{Place, Source};

pub(crate) use boundary::Count;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Budget {
    tests: Count,
    files: Count,
    lines: Count,
    test_lines: Count,
    starts: Count,
}

#[derive(Debug, Default)]
pub(crate) struct Budgets(BTreeMap<Name, Budget>);

#[derive(Debug, Default)]
struct TestScan {
    tests: Count,
    test_lines: Count,
    gated: Count,
}

impl Budgets {
    const PROPERTY_BLOCK: &'static str = "proptest";

    pub(crate) fn of_file(source: &Source, file: &syn::File, gate: Gate) -> Budgets {
        let mut scan = TestScan::default();
        scan.visit_file(file);
        let lines = Count::new(source.text().as_ref().bytes().filter(|byte| *byte == b'\n').count());
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped);
        let budget = Budget {
            tests: scan.tests,
            files: Count::zero().incremented(),
            lines,
            test_lines: shipped.map_or(lines, |()| scan.test_lines),
            starts: gate.entered().map_or_else(Count::zero, |()| ProcessStarters::everywhere(file)),
        };
        Budgets(BTreeMap::from([(source.package().crate_name(), budget)]))
    }

    pub(crate) fn joined(mut self, other: Budgets) -> Budgets {
        other.0.into_iter().for_each(|(name, budget)| {
            let total = self.0.get(&name).copied().unwrap_or_default() + budget;
            self.0.insert(name, total);
        });
        self
    }

    pub(crate) fn written(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "test budget, report only:")?;
        self.0.iter().try_for_each(|(name, budget)| writeln!(out, "{name}: {budget}"))
    }
}

impl<'ast> Visit<'ast> for TestScan {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.tests = self.tests.plus(Count::total(TestMark::among(&function.attrs).map(|_| Count::new(1)).into_iter()));
        visit::visit_item_fn(self, function);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Gate::of(&module.attrs) {
            Gate::Entered => visit::visit_item_mod(self, module),
            Gate::Skipped => {
                let span = module.span();
                let outermost =
                    (self.gated == Count::zero()).then(|| Count::new(span.end().line - span.start().line + 1));
                self.test_lines = self.test_lines.plus(outermost.unwrap_or_default());
                let outer = self.gated;
                self.gated = outer.incremented();
                visit::visit_item_mod(self, module);
                self.gated = outer;
            }
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let property = invocation.path.segments.last().filter(|segment| segment.ident == Budgets::PROPERTY_BLOCK);
        self.tests =
            self.tests.plus(property.map_or_else(Count::zero, |_| TestMark::counted(invocation.tokens.clone())));
    }
}

impl Add for Budget {
    type Output = Budget;

    fn add(self, other: Budget) -> Budget {
        Budget {
            tests: self.tests.plus(other.tests),
            files: self.files.plus(other.files),
            lines: self.lines.plus(other.lines),
            test_lines: self.test_lines.plus(other.test_lines),
            starts: self.starts.plus(other.starts),
        }
    }
}

impl fmt::Display for Budget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} tests in {} files, {} of {} lines, {} process starts",
            self.tests, self.files, self.test_lines, self.lines, self.starts
        )
    }
}
