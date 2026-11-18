use algebra::{Monoid, Semigroup};
use lint_layout::Failure;
use lint_roots::Survey;

use super::modules::{Child, Sought};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Module,
    Test,
}

#[derive(Debug)]
pub(super) enum Resolution {
    Modules(Vec<Child>),
    Tests(Vec<Child>),
    Failed(Failure),
}

#[derive(Debug, Default)]
pub(super) struct Declared {
    modules: Vec<Child>,
    tests: Vec<Child>,
    failures: Vec<Failure>,
}

impl Semigroup for Declared {
    fn combine(self, other: Declared) -> Declared {
        Declared {
            modules: self.modules.combine(other.modules),
            tests: self.tests.combine(other.tests),
            failures: self.failures.combine(other.failures),
        }
    }
}

impl Monoid for Declared {}

impl From<Resolution> for Declared {
    fn from(resolution: Resolution) -> Declared {
        match resolution {
            Resolution::Modules(modules) => Declared { modules, ..Declared::default() },
            Resolution::Tests(tests) => Declared { tests, ..Declared::default() },
            Resolution::Failed(failure) => Declared { failures: vec![failure], ..Declared::default() },
        }
    }
}

impl Declared {
    pub(super) fn resolved(sought: &[Sought], survey: &Survey) -> Declared {
        Monoid::concat(sought.iter().filter_map(|declaration| declaration.resolution(survey)).map(Declared::from))
    }

    pub(super) fn children(&self, kind: Kind) -> &[Child] {
        match kind {
            Kind::Module => &self.modules,
            Kind::Test => &self.tests,
        }
    }

    pub(super) fn failures(self) -> Vec<Failure> {
        self.failures
    }
}
