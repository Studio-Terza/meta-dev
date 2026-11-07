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

impl TestShare {
    const fn test_lines(self, inline: Count, whole: Count) -> Count {
        match self {
            Self::InlineModules => inline,
            Self::WholeFile => whole,
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

    fn measured(file: &SourceFile, share: TestShare) -> Self {
        let lines = Count::new(file.text().as_ref().bytes().filter(|byte| char::from(*byte) == Self::LINE_END).count());
        let scan = TestScan::measured(file.syntax());
        Self {
            files: Count::new(1),
            lines,
            starts: Count::new(CommandPaths::across(std::slice::from_ref(file), Scope::Everywhere).len()),
            test_lines: share.test_lines(scan.test_lines, lines),
            ..scan
        }
    }
}

impl From<&SourceFile> for CrateBudget {
    fn from(file: &SourceFile) -> Self {
        let share = file.shipped().map_or(TestShare::WholeFile, |_| TestShare::InlineModules);
        file.test_modules()
            .iter()
            .map(|module| Self::measured(module, TestShare::WholeFile))
            .fold(Self::measured(file, share), Add::add)
    }
}

impl Add for CrateBudget {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
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
