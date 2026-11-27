use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Write};
use std::ops::Add;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use crate::gate::{Gate, TestMark};
use crate::lints::ProcessStarters;
use crate::source::{Place, Source};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Budget {
    tests: usize,
    files: usize,
    lines: usize,
    test_lines: usize,
    starts: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Budgets(BTreeMap<String, Budget>);

#[derive(Debug, Default)]
struct TestScan {
    tests: usize,
    test_lines: usize,
    gated: usize,
}

impl Budgets {
    const PROPERTY_BLOCK: &'static str = "proptest";

    pub(crate) fn of_file(source: &Source, file: &syn::File, gate: Gate) -> Budgets {
        let mut scan = TestScan::default();
        scan.visit_file(file);
        let lines = source.text().bytes().filter(|byte| *byte == b'\n').count();
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped);
        let budget = Budget {
            tests: scan.tests,
            files: 1,
            lines,
            test_lines: shipped.map_or(lines, |()| scan.test_lines),
            starts: ProcessStarters::everywhere(file),
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
        self.tests += TestMark::among(&function.attrs).map_or(0, |_| 1);
        visit::visit_item_fn(self, function);
    }

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

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let property = invocation.path.segments.last().filter(|segment| segment.ident == Budgets::PROPERTY_BLOCK);
        self.tests += property.map_or(0, |_| TestMark::counted(invocation.tokens.clone()));
    }
}

impl Add for Budget {
    type Output = Budget;

    fn add(self, other: Budget) -> Budget {
        Budget {
            tests: self.tests + other.tests,
            files: self.files + other.files,
            lines: self.lines + other.lines,
            test_lines: self.test_lines + other.test_lines,
            starts: self.starts + other.starts,
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
