use lint_core::{SourceFile, Violation};
use lint_sweep::Finds;

pub type Start = for<'file> fn(&'file SourceFile) -> Box<dyn Finds<'file> + 'file>;

pub type Scan = for<'file> fn(&'file SourceFile) -> Vec<Violation<'file>>;

#[derive(Clone, Copy, Debug)]
pub enum Lead {
    Swept(Start),
    Filed(Scan),
}

impl Lead {
    #[must_use]
    pub fn swept(self) -> Option<Start> {
        match self {
            Lead::Swept(start) => Some(start),
            Lead::Filed(_) => None,
        }
    }

    #[must_use]
    pub fn scanned(self, file: &SourceFile) -> Option<Vec<Violation<'_>>> {
        match self {
            Lead::Filed(scan) => Some(scan(file)),
            Lead::Swept(_) => None,
        }
    }
}

pub trait Leader: 'static {
    const LEAD: Lead;
}
