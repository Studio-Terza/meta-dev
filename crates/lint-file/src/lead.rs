use lint_core::{SourceFile, Violation};
use lint_sweep::{Finds, Member, Sweep};

pub type Start = for<'file> fn(&'file SourceFile) -> Box<dyn Finds<'file> + 'file>;

pub type SortStart = for<'file> fn(&'file SourceFile) -> Box<dyn Finds<'file, Vec<Vec<Violation<'file>>>> + 'file>;

pub type Scan = for<'file> fn(&'file SourceFile) -> Vec<Violation<'file>>;

pub type SortedScan = for<'file> fn(&'file SourceFile) -> Vec<Vec<Violation<'file>>>;

#[derive(Clone, Copy, Debug)]
pub enum Lead {
    Swept(Start),
    Sorting(SortStart),
    Scanned(SortedScan),
    Filed(Scan),
}

impl Lead {
    #[must_use]
    pub fn swept(self) -> Option<Start> {
        match self {
            Lead::Swept(start) => Some(start),
            Lead::Sorting(_) | Lead::Scanned(_) | Lead::Filed(_) => None,
        }
    }

    #[must_use]
    pub fn sorting(self) -> Option<SortStart> {
        match self {
            Lead::Sorting(start) => Some(start),
            Lead::Swept(_) | Lead::Scanned(_) | Lead::Filed(_) => None,
        }
    }

    #[must_use]
    pub fn scanned(self, file: &SourceFile) -> Option<Vec<Vec<Violation<'_>>>> {
        match self {
            Lead::Scanned(scan) => Some(scan(file)),
            Lead::Filed(scan) => Some(vec![scan(file)]),
            Lead::Swept(_) | Lead::Sorting(_) => None,
        }
    }

    #[must_use]
    pub fn member_of(self, file: &SourceFile, member: Member) -> Vec<Violation<'_>> {
        let lists = self.scanned(file).unwrap_or_else(|| {
            let mut finds: Vec<Box<dyn Finds<'_> + '_>> = self.swept().map(|start| start(file)).into_iter().collect();
            let mut sorts: Vec<Box<dyn Finds<'_, Vec<Vec<Violation<'_>>>> + '_>> =
                self.sorting().map(|start| start(file)).into_iter().collect();
            Sweep::over_leads(file, &mut finds, &mut sorts);
            let found = finds.into_iter().map(|watcher| vec![watcher.found()]);
            found.chain(sorts.into_iter().map(Finds::found)).next().unwrap_or_default()
        });
        lists.into_iter().nth(member.index()).unwrap_or_default()
    }
}

pub trait Leader: 'static {
    const LEAD: Lead;
}
