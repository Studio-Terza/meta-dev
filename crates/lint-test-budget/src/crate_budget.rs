mod test_scan;

use std::fmt;
use std::ops::Add;

use lint_core::SourceFile;

use crate::count::Count;
use test_scan::TestScan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestShare {
    InlineModules,
    WholeFile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrateBudget {
    tests: Count,
    files: Count,
    lines: Count,
    test_lines: Count,
}

impl CrateBudget {
    const LINE_END: char = '\n';

    #[must_use]
    pub fn new(tests: Count, files: Count, lines: Count, test_lines: Count) -> CrateBudget {
        CrateBudget { tests, files, lines, test_lines }
    }

    fn measured(file: &SourceFile, share: TestShare) -> CrateBudget {
        let lines = Count::from(file.text().bytes().filter(|byte| char::from(*byte) == Self::LINE_END).count());
        let scan = TestScan::measured(file.syntax());
        CrateBudget {
            files: Count::from(1),
            lines,
            test_lines: match share {
                TestShare::InlineModules => scan.test_lines,
                TestShare::WholeFile => lines,
            },
            ..scan
        }
    }
}

impl From<&SourceFile> for CrateBudget {
    fn from(file: &SourceFile) -> CrateBudget {
        let share = if file.is_shipped() { TestShare::InlineModules } else { TestShare::WholeFile };
        file.test_modules()
            .iter()
            .map(|module| Self::measured(module, TestShare::WholeFile))
            .fold(Self::measured(file, share), Add::add)
    }
}

impl Add for CrateBudget {
    type Output = CrateBudget;

    fn add(self, other: CrateBudget) -> CrateBudget {
        CrateBudget {
            tests: self.tests + other.tests,
            files: self.files + other.files,
            lines: self.lines + other.lines,
            test_lines: self.test_lines + other.test_lines,
        }
    }
}

impl fmt::Display for CrateBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} tests in {} files, {} of {} lines", self.tests, self.files, self.test_lines, self.lines)
    }
}
