mod count;
mod test_scan;

use std::fmt;
use std::ops::Add;

use lint_core::SourceFile;
use lint_sweep::{Gathers, Sweep, Watching};
use newtype::Counter;

use count::Count;
use test_scan::TestScan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestShare {
    InlineModules,
    WholeFile,
}

impl TestShare {
    fn test_lines(self, inline: Count, whole: Count) -> Count {
        match self {
            TestShare::InlineModules => inline,
            TestShare::WholeFile => whole,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrateBudget {
    tests: Count,
    files: Count,
    lines: Count,
    test_lines: Count,
    starts: Count,
}

impl CrateBudget {
    const LINE_END: char = '\n';

    fn measured(file: &SourceFile, share: TestShare, scan: CrateBudget) -> CrateBudget {
        let lines = Count::new(file.text().as_ref().bytes().filter(|byte| char::from(*byte) == Self::LINE_END).count());
        CrateBudget { files: Count::new(1), lines, test_lines: share.test_lines(scan.test_lines, lines), ..scan }
    }

    pub(crate) fn start() -> CrateBudget {
        CrateBudget { starts: Count::new(1), ..CrateBudget::default() }
    }
}

#[derive(Debug)]
pub struct BudgetGathering {
    file: CrateBudget,
    share: TestShare,
    tested: CrateBudget,
    scan: TestScan,
}

impl From<&SourceFile> for BudgetGathering {
    fn from(file: &SourceFile) -> BudgetGathering {
        let share = file.shipped().map_or(TestShare::WholeFile, |_| TestShare::InlineModules);
        let tested = file.test_modules().iter().map(|module| {
            let scan = Sweep::gathered(module, TestScan::default());
            CrateBudget::measured(module, TestShare::WholeFile, scan)
        });
        let unscanned = CrateBudget::measured(file, share, CrateBudget::default());
        BudgetGathering {
            file: unscanned,
            share,
            tested: tested.fold(CrateBudget::default(), Add::add),
            scan: TestScan::default(),
        }
    }
}

impl Gathers<CrateBudget> for BudgetGathering {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(&mut self.scan)
    }

    fn gathered(self) -> CrateBudget {
        let share = self.share;
        let unscanned = self.file;
        let file = self.scan.counted(|tests, test_lines| CrateBudget {
            test_lines: share.test_lines(test_lines, unscanned.lines),
            tests,
            ..unscanned
        });
        file + self.tested
    }
}

impl Gathers<CrateBudget> for TestScan {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> CrateBudget {
        self.counted(|tests, _| CrateBudget { tests, ..CrateBudget::default() })
    }
}

impl Add for CrateBudget {
    type Output = CrateBudget;

    fn add(self, other: CrateBudget) -> CrateBudget {
        CrateBudget {
            tests: self.tests.plus(other.tests),
            files: self.files.plus(other.files),
            lines: self.lines.plus(other.lines),
            test_lines: self.test_lines.plus(other.test_lines),
            starts: self.starts.plus(other.starts),
        }
    }
}

impl fmt::Display for CrateBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} tests in {} files, {} of {} lines, {} process starts",
            self.tests, self.files, self.test_lines, self.lines, self.starts
        )
    }
}
