mod count;
mod test_scan;

use std::fmt;
use std::ops::Add;

use lint_core::SourceFile;
use lint_process_starters::{CommandPaths, Scope};
use newtype::Counter;

use count::Count;
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
    starts: Count,
}

impl CrateBudget {
    const LINE_END: char = '\n';

    fn measured(file: &SourceFile, share: TestShare) -> CrateBudget {
        let lines = Count::new(file.text().as_ref().bytes().filter(|byte| char::from(*byte) == Self::LINE_END).count());
        let scan = TestScan::measured(file.syntax());
        CrateBudget {
            files: Count::new(1),
            lines,
            starts: Count::new(CommandPaths::found(file, Scope::Everywhere).len()),
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
