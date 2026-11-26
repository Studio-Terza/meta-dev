use lint_core::{SourceFile, Violation};
use lint_sweep::{Found, Visited};

#[derive(Debug)]
pub struct Led<'file> {
    violations: Vec<Violation<'file>>,
    visited: Option<Visited>,
}

impl<'file> Led<'file> {
    #[must_use]
    pub fn visited(&self) -> Option<Visited> {
        self.visited
    }

    pub fn taken(&mut self) -> Vec<Violation<'file>> {
        std::mem::take(&mut self.violations)
    }
}

impl<'file> From<Found<'file>> for Led<'file> {
    fn from(found: Found<'file>) -> Led<'file> {
        let visited = Some(found.visited());
        Led { violations: found.into_iter().collect(), visited }
    }
}

impl<'file> From<Vec<Violation<'file>>> for Led<'file> {
    fn from(violations: Vec<Violation<'file>>) -> Led<'file> {
        Led { violations, visited: None }
    }
}

pub trait Leader: 'static {
    fn led(file: &SourceFile) -> Led<'_>;
}
