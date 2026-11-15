use lint_core::{SourceFile, Violation};
use lint_sweep::{Finds, Member, Sweep};

pub type Start = for<'file> fn(&'file SourceFile) -> Box<dyn Finds<'file> + 'file>;

pub type Scan = for<'file> fn(&'file SourceFile) -> Vec<Violation<'file>>;

pub type SortedScan = for<'file> fn(&'file SourceFile) -> Vec<Vec<Violation<'file>>>;

#[derive(Clone, Copy, Debug)]
pub enum Lead {
    Swept(Start),
    Scanned(SortedScan),
    Filed(Scan),
}

impl Lead {
    #[must_use]
    pub fn swept(self) -> Option<Start> {
        match self {
            Lead::Swept(start) => Some(start),
            Lead::Scanned(_) | Lead::Filed(_) => None,
        }
    }

    #[must_use]
    pub fn scanned(self, file: &SourceFile) -> Option<Vec<Vec<Violation<'_>>>> {
        match self {
            Lead::Scanned(scan) => Some(scan(file)),
            Lead::Filed(scan) => Some(vec![scan(file)]),
            Lead::Swept(_) => None,
        }
    }

    fn watched(file: &SourceFile, start: Start) -> Vec<Vec<Violation<'_>>> {
        let mut watchers = [vec![start(file)]];
        Sweep::over_all(file, &mut watchers);
        let [only] = watchers;
        only.into_iter().flat_map(Finds::sorted).collect()
    }

    #[must_use]
    pub fn member_of(self, file: &SourceFile, member: Member) -> Vec<Violation<'_>> {
        let sorted = self.scanned(file).or_else(|| self.swept().map(|start| Self::watched(file, start)));
        sorted.unwrap_or_default().into_iter().nth(member.index()).unwrap_or_default()
    }
}

pub trait Leader: 'static {
    const LEAD: Lead;
}
