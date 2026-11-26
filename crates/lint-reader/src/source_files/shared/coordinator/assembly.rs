use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use lint_core::Joins;
use lint_layout::Failure;
use lint_placement::CrateFile;
use lint_roots::Survey;

use crate::source_files::Stage;
use crate::source_files::declared::{Declared, Kind};
use crate::source_files::located::{Home, Located};
use crate::source_files::modules::Child;
use crate::source_files::shared::boundary::Partition;
use crate::source_files::shared::protocol::Main;
use crate::source_files::staged::Staged;

pub(super) type Outcomes<'tree> = BTreeMap<Located<'tree>, Result<Declared, Failure>>;

#[derive(Debug)]
pub(super) enum Planned<'tree> {
    Main(Main<'tree>),
    Failed(Located<'tree>),
}

#[derive(Debug, Default)]
pub(super) struct Assembled<'tree> {
    mains: BTreeMap<Partition, Vec<Main<'tree>>>,
    failures: Vec<Failure>,
}

impl Joins for Assembled<'_> {
    fn combine(self, other: Self) -> Self {
        let mut mains = self.mains;
        other.mains.into_iter().for_each(|(hand, found)| mains.entry(hand).or_default().extend(found));
        Assembled { mains, failures: self.failures.combine(other.failures) }
    }
}

impl<'tree> Assembled<'tree> {
    pub(super) fn concluded<Out>(
        self,
        into: impl FnOnce(BTreeMap<Partition, Vec<Main<'tree>>>, Vec<Failure>) -> Out,
    ) -> Out {
        into(self.mains, self.failures)
    }
}

impl<'tree> AsRef<Located<'tree>> for Planned<'tree> {
    fn as_ref(&self) -> &Located<'tree> {
        match self {
            Planned::Main(main) => main.as_ref(),
            Planned::Failed(located) => located,
        }
    }
}

impl<'tree> Planned<'tree> {
    pub(super) fn taken(
        self,
        outcome: Option<Result<Declared, Failure>>,
        hands: &BTreeMap<Located<'tree>, Partition>,
    ) -> Assembled<'tree> {
        match self {
            Planned::Main(main) => Planned::main(main, outcome, hands),
            Planned::Failed(_) => Assembled { failures: Planned::failed(outcome), ..Assembled::default() },
        }
    }

    fn main(
        main: Main<'tree>,
        outcome: Option<Result<Declared, Failure>>,
        hands: &BTreeMap<Located<'tree>, Partition>,
    ) -> Assembled<'tree> {
        let hand = hands.get(main.as_ref()).copied().unwrap_or(Partition::MAIN);
        let failures = outcome.into_iter().flatten().flat_map(Declared::failures).collect();
        Assembled { mains: BTreeMap::from([(hand, vec![main])]), failures }
    }

    fn failed(outcome: Option<Result<Declared, Failure>>) -> Vec<Failure> {
        match outcome {
            Some(Err(failure)) => vec![failure],
            Some(Ok(_)) | None => Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct Plan<'tree> {
    order: Vec<Planned<'tree>>,
    seen: BTreeSet<PathBuf>,
}

impl Joins for Plan<'_> {
    fn combine(self, other: Self) -> Self {
        let mut seen = self.seen;
        seen.extend(other.seen);
        Plan { order: self.order.combine(other.order), seen }
    }
}

#[derive(Clone, Copy, Debug)]
struct Marks<'up, 'tree> {
    plan: &'up Plan<'tree>,
    up: Option<&'up Marks<'up, 'tree>>,
}

impl Marks<'_, '_> {
    fn seen(self, path: &Path) -> Option<()> {
        self.plan.seen.get(path).map(|_| ()).or_else(|| self.up.and_then(|up| up.seen(path)))
    }
}

#[derive(Debug, Default)]
struct Tested<'tree> {
    found: Vec<Located<'tree>>,
    consumed: BTreeSet<Located<'tree>>,
}

impl Joins for Tested<'_> {
    fn combine(self, other: Self) -> Self {
        let mut consumed = self.consumed;
        consumed.extend(other.consumed);
        Tested { found: self.found.combine(other.found), consumed }
    }
}


#[derive(Clone, Copy, Debug)]
struct TestMarks<'up, 'tree> {
    tested: &'up Tested<'tree>,
    up: Option<&'up TestMarks<'up, 'tree>>,
}

impl<'tree> TestMarks<'_, 'tree> {
    fn consumed(self, located: &Located<'tree>) -> Option<()> {
        self.tested.consumed.get(located).map(|_| ()).or_else(|| self.up.and_then(|up| up.consumed(located)))
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Planner<'plan, 'tree> {
    outcomes: &'plan Outcomes<'tree>,
    survey: &'plan Survey,
}

#[derive(Clone, Copy, Debug)]
struct Rooted<'plan, 'tree> {
    outcomes: &'plan Outcomes<'tree>,
    survey: &'plan Survey,
    home: Home<'plan, 'tree>,
    crate_file: &'plan CrateFile,
}

impl<'plan, 'tree> Planner<'plan, 'tree> {
    pub(super) fn new(outcomes: &'plan Outcomes<'tree>, survey: &'plan Survey) -> Planner<'plan, 'tree> {
        Planner { outcomes, survey }
    }

    pub(super) fn planned(self, stage: &'tree Stage<'tree>) -> Vec<Planned<'tree>> {
        let targets = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().flat_map(Staged::targets).map(move |target| (tree, target)));
        let roots = targets.flat_map(|(tree, target)| {
            target.files().iter().map(move |file| (Home::new(tree, target.package()), tree.display(file), file))
        });
        let plan = roots.fold(Plan::default(), |plan, (home, shown, file)| {
            let crate_file = CrateFile::from(shown);
            let rooted = Rooted { outcomes: self.outcomes, survey: self.survey, home, crate_file: &crate_file };
            let next = rooted.file(file, Marks { plan: &plan, up: None });
            plan.combine(next)
        });
        plan.order
    }
}

impl<'tree> Rooted<'_, 'tree> {
    fn file(self, path: &Path, marks: Marks<'_, 'tree>) -> Plan<'tree> {
        let unseen = marks.seen(path).map_or(Some(()), |()| None);
        unseen.map_or_default(|()| {
            let seeing = Plan { seen: BTreeSet::from([path.to_path_buf()]), ..Plan::default() };
            let rest = self.fresh(&self.home.located(path), Marks { plan: &seeing, up: Some(&marks) });
            seeing.combine(rest)
        })
    }

    fn fresh(self, located: &Located<'tree>, marks: Marks<'_, 'tree>) -> Plan<'tree> {
        let found = self.outcomes.get(located);
        let declared = found.and_then(|outcome| outcome.iter().next());
        let failed = found.filter(|outcome| outcome.is_err()).map(|_| Planned::Failed(located.clone()));
        let rest = declared.map_or_default(|done| self.declared(located.clone(), done, marks));
        let failing = Plan { order: failed.into_iter().collect(), ..Plan::default() };
        failing.combine(rest)
    }

    fn declared(self, located: Located<'tree>, declared: &Declared, marks: Marks<'_, 'tree>) -> Plan<'tree> {
        let tested = Testing { rooted: self }.tests(declared.children(Kind::Test));
        let main = Plan {
            order: vec![Planned::Main(Main::new(located, self.crate_file.clone(), tested.found))],
            ..Plan::default()
        };
        let after = Marks { plan: &main, up: Some(&marks) };
        let modules = declared.children(Kind::Module).iter().fold(Plan::default(), |plan, module| {
            let next = self.file(module.canonical(self.survey), Marks { plan: &plan, up: Some(&after) });
            plan.combine(next)
        });
        main.combine(modules)
    }
}

#[derive(Clone, Copy, Debug)]
struct Testing<'plan, 'tree> {
    rooted: Rooted<'plan, 'tree>,
}

impl<'tree> Testing<'_, 'tree> {
    fn tests(self, tests: &[Child]) -> Tested<'tree> {
        tests.iter().fold(Tested::default(), |tested, test| {
            let marks = TestMarks { tested: &tested, up: None };
            let next = self.file(test.canonical(self.rooted.survey), marks);
            tested.combine(next)
        })
    }

    fn file(self, path: &Path, marks: TestMarks<'_, 'tree>) -> Tested<'tree> {
        self.fresh(&self.rooted.home.located(path), marks)
    }

    fn fresh(self, located: &Located<'tree>, marks: TestMarks<'_, 'tree>) -> Tested<'tree> {
        let found = marks.consumed(located).map_or_else(|| self.rooted.outcomes.get(located), |()| None);
        let declared = found.and_then(|outcome| outcome.iter().next());
        let own = Tested {
            found: declared.map(|_| located.clone()).into_iter().collect(),
            consumed: found.map(|_| located.clone()).into_iter().collect(),
        };
        let inner = TestMarks { tested: &own, up: Some(&marks) };
        let nested =
            declared.into_iter().flat_map(|done| done.children(Kind::Module).iter().chain(done.children(Kind::Test)));
        let below = nested.fold(Tested::default(), |tested, child| {
            let next = self.file(child.canonical(self.rooted.survey), TestMarks { tested: &tested, up: Some(&inner) });
            tested.combine(next)
        });
        own.combine(below)
    }
}
