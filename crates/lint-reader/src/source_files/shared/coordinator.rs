mod assembly;

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::Path;
use std::sync::OnceLock;

use file_ports::Files;
use lint_core::Joins;
use lint_core::{FileHead, Readout, RepoFile, SourceFile};
use lint_roots::Survey;

use super::boundary::{Hands, Partition};
use super::protocol::{Assigned, Assignment, Due, Parsed, Reads, Report, Sighting};
use super::reader::{Assembling, Dispatched, Linked, Reader, Remote};
use crate::source_files::Stage;
use crate::source_files::declared::{Declared, Kind};
use crate::source_files::ignored::Ignored;
use crate::source_files::located::{Located, Node};
use crate::source_files::staged::Staged;
use assembly::{Assembled, Outcomes, Planner};
use invariants::Ignoring;

#[derive(Debug)]
pub struct SharedRead<'tree> {
    stage: &'tree Stage<'tree>,
    table: OnceLock<Vec<FileHead>>,
}

#[derive(Debug)]
pub struct Crew<'shared, 'tree> {
    coordinator: Coordinator<'shared, 'tree>,
    readers: Vec<Reader<'tree>>,
}

#[must_use]
#[derive(Debug)]
pub struct Coordinator<'shared, 'tree> {
    shared: &'shared SharedRead<'tree>,
    remotes: Vec<Remote<'tree>>,
}

impl<'tree> SharedRead<'tree> {
    #[must_use]
    pub fn new(stage: &'tree Stage<'tree>) -> SharedRead<'tree> {
        SharedRead { stage, table: OnceLock::new() }
    }

    pub fn crew(&self, helpers: impl IntoIterator<Item = Partition>) -> Crew<'_, 'tree> {
        let linked = helpers.into_iter().map(|hand| Linked::new(&self.stage.packages, hand));
        let (readers, remotes): (Vec<Reader<'tree>>, Vec<Remote<'tree>>) =
            linked.map(|link| link.concluded(|reader, remote| (reader, remote))).unzip();
        Crew { coordinator: Coordinator { shared: self, remotes }, readers }
    }

    #[must_use]
    pub fn heads(&self) -> &[FileHead] {
        self.table.wait()
    }
}

impl Drop for Coordinator<'_, '_> {
    fn drop(&mut self) {
        self.shared.table.get_or_init(Vec::new);
    }
}

impl<'shared, 'tree> Crew<'shared, 'tree> {
    pub fn concluded<Out>(self, into: impl FnOnce(Coordinator<'shared, 'tree>, Vec<Reader<'tree>>) -> Out) -> Out {
        into(self.coordinator, self.readers)
    }
}

#[derive(Debug, Default)]
struct Gathered<'tree> {
    outcomes: Outcomes<'tree>,
    hands: BTreeMap<Located<'tree>, Partition>,
    kept: BTreeMap<Located<'tree>, SourceFile>,
}

impl Joins for Gathered<'_> {
    fn combine(self, other: Self) -> Self {
        let mut outcomes = self.outcomes;
        outcomes.extend(other.outcomes);
        let mut hands = self.hands;
        hands.extend(other.hands);
        let mut kept = self.kept;
        kept.extend(other.kept);
        Gathered { outcomes, hands, kept }
    }
}

#[derive(Debug)]
struct Explored<'tree> {
    gathered: Gathered<'tree>,
    survey: Survey,
}

#[derive(Debug)]
struct Level<'tree> {
    due: Vec<Due<'tree>>,
    seen: BTreeSet<Located<'tree>>,
}

#[derive(Debug)]
struct Dealt<'tree> {
    assigned: Vec<Assigned<'tree>>,
    seen: BTreeSet<Located<'tree>>,
}

impl<'tree> Level<'tree> {
    fn started(starts: Vec<Node<'tree>>) -> Level<'tree> {
        Level::after(BTreeSet::new(), starts.into_iter().map(|node| Due::new(node, None)))
    }

    fn after(seen: BTreeSet<Located<'tree>>, found: impl Iterator<Item = Due<'tree>>) -> Level<'tree> {
        let mut visited = seen;
        let due = found.filter(|next| visited.insert(AsRef::<Located>::as_ref(next).clone())).collect();
        Level { due, seen: visited }
    }

    fn dealt(self, hands: &Hands, files: &impl Files) -> Dealt<'tree> {
        let weighed = |due: &Due<'tree>| {
            files
                .size(AsRef::<Path>::as_ref(AsRef::<Located>::as_ref(due)))
                .ignoring(Ignored::Unsized)
                .unwrap_or_default()
        };
        Dealt { assigned: hands.dealt(self.due, weighed), seen: self.seen }
    }
}

#[derive(Debug, Default)]
struct Sighted<'tree> {
    sightings: Vec<Sighting<'tree>>,
    survey: Survey,
    kept: BTreeMap<Located<'tree>, SourceFile>,
}

impl Reads for Coordinator<'_, '_> {
    fn read(self, files: &impl Files) -> Readout {
        let stage = self.shared.stage;
        let starts = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().flat_map(|entry| entry.starts(tree)));
        let hands: Hands = iter::once(Partition::MAIN).chain(self.remotes.iter().map(Remote::hand)).collect();
        let exploring = Exploring { coordinator: &self, files };
        let explored = exploring.explored(Level::started(starts.collect()), Survey::default(), &hands);
        let planned = Planner::new(&explored.gathered.outcomes, &explored.survey).planned(stage);
        let Gathered { outcomes, hands: placed, kept } = explored.gathered;
        let taken = planned.into_iter().scan(outcomes, |owned, found| {
            let outcome = owned.remove(found.as_ref());
            Some(found.taken(outcome, &placed))
        });
        let repo_files: Vec<RepoFile> = stage.staged.iter().flatten().flat_map(Staged::repo_files).cloned().collect();
        <Assembled<'_> as Joins>::concat(taken).concluded(|mains, failures| {
            let mut assigned = mains;
            let assembling: Vec<_> = self
                .remotes
                .iter()
                .filter_map(|remote| {
                    let mine = assigned.remove(&remote.hand()).unwrap_or_default();
                    remote.assigned(Assignment::new(mine, repo_files.clone(), Vec::new()))
                })
                .collect();
            let own = assigned.remove(&Partition::MAIN).unwrap_or_default();
            let readout = Assignment::new(own, repo_files, failures).readout(kept);
            let mut heads: Vec<FileHead> = readout.files().iter().map(|file| file.head().clone()).collect();
            heads.extend(assembling.into_iter().flat_map(Assembling::heads));
            heads.sort_by(|earlier, later| AsRef::<Path>::as_ref(earlier).cmp(AsRef::<Path>::as_ref(later)));
            self.shared.table.get_or_init(|| heads);
            readout
        })
    }
}

#[derive(Debug)]
struct Exploring<'run, 'shared, 'tree, Port> {
    coordinator: &'run Coordinator<'shared, 'tree>,
    files: &'run Port,
}

impl<'tree, Port: Files> Exploring<'_, '_, 'tree, Port> {
    fn explored(&self, level: Level<'tree>, survey: Survey, hands: &Hands) -> Explored<'tree> {
        match level.due.first() {
            Some(_) => self.levelled(level.dealt(hands, self.files), survey, hands),
            None => Explored { gathered: Gathered::default(), survey },
        }
    }

    fn levelled(&self, dealt: Dealt<'tree>, survey: Survey, hands: &Hands) -> Explored<'tree> {
        let assigned = dealt.assigned;
        let sighted = self.sighted(&assigned);
        let learned = survey.combine(sighted.survey);
        let known = &learned;
        let outcomes: Outcomes<'tree> = sighted
            .sightings
            .into_iter()
            .map(|sighting| {
                let located = AsRef::<Located>::as_ref(&sighting).clone();
                (located, sighting.sought().map(|sought| Declared::resolved(&sought, known)))
            })
            .collect();
        let next = assigned.iter().flat_map(|found| {
            let hand = found.hand();
            let home = found.node().home();
            let declared = outcomes.get(found.node().located()).into_iter().flatten();
            let modules = declared.clone().flat_map(|done| done.children(Kind::Module));
            let tests = declared.flat_map(|done| done.children(Kind::Test));
            let owned = modules.map(move |module| Due::new(home.child(module, known), None));
            owned.chain(tests.map(move |test| Due::new(home.child(test, known), Some(hand))))
        });
        let following = Level::after(dealt.seen, next);
        let hands_of = assigned.iter().map(|found| (found.node().located().clone(), found.hand())).collect();
        let deeper = self.explored(following, learned, hands);
        let own = Gathered { outcomes, hands: hands_of, kept: sighted.kept };
        Explored { gathered: own.combine(deeper.gathered), survey: deeper.survey }
    }

    fn sighted(&self, assigned: &[Assigned<'tree>]) -> Sighted<'tree> {
        let grouped: BTreeMap<Partition, Vec<Node<'tree>>> =
            assigned.iter().fold(BTreeMap::new(), |mut by: BTreeMap<Partition, Vec<Node<'tree>>>, found| {
                by.entry(found.hand()).or_default().push(found.node().clone());
                by
            });
        let mut by_hand = grouped;
        let waiting: Vec<Dispatched<'_, 'tree>> = self
            .coordinator
            .remotes
            .iter()
            .filter_map(|remote| {
                let nodes = by_hand.remove(&remote.hand()).unwrap_or_default();
                nodes.first()?;
                remote.dispatched(nodes)
            })
            .collect();
        let local: Vec<Node<'tree>> = by_hand.remove(&Partition::MAIN).unwrap_or_default();
        let packages = &self.coordinator.shared.stage.packages;
        Parsed::read(self.files, packages, &local, |own, kept| {
            let report: Report<'tree> =
                Joins::concat(iter::once(own).chain(waiting.into_iter().map(Dispatched::sighted)));
            report.concluded(|sightings, survey| Sighted { sightings, survey, kept })
        })
    }
}
