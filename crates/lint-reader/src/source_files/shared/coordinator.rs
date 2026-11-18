mod assembly;
mod tide;

use std::collections::BTreeMap;
use std::iter;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver};

use algebra::{Monoid, Semigroup};
use file_ports::{Files, Size};
use lint_core::{FileHead, Readout, RepoFile};

use super::boundary::{Flights, Partition, Rank};
use super::protocol::{Assignment, Parsed, Reads, Reply, Report};
use super::reader::{Linked, Reader, Remote};
use crate::source_files::Stage;
use crate::source_files::declared::{Declared, Kind};
use crate::source_files::located::{Located, Node};
use assembly::{Assembled, Planner};
use tide::{Ebb, Explored, Key, Lineage, Offer, Settled, Surge, Tide};

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

#[derive(Debug)]
pub struct Coordinator<'shared, 'tree> {
    shared: &'shared SharedRead<'tree>,
    remotes: Vec<Remote<'tree>>,
    replies: Receiver<Reply<'tree>>,
}

impl<'tree> SharedRead<'tree> {
    #[must_use]
    pub fn new(stage: &'tree Stage<'tree>) -> SharedRead<'tree> {
        SharedRead { stage, table: OnceLock::new() }
    }

    pub fn crew(&self, helpers: impl IntoIterator<Item = Partition>) -> Crew<'_, 'tree> {
        let (sender, replies) = mpsc::channel();
        let linked = helpers.into_iter().map(|hand| Linked::new(&self.stage.packages, hand, sender.clone()));
        let (readers, remotes): (Vec<Reader<'tree>>, Vec<Remote<'tree>>) =
            linked.map(|link| link.concluded(|reader, remote| (reader, remote))).unzip();
        Crew { coordinator: Coordinator { shared: self, remotes, replies }, readers }
    }

    #[must_use]
    pub fn heads(&self) -> &[FileHead] {
        self.table.wait()
    }
}

impl<'shared, 'tree> Crew<'shared, 'tree> {
    pub fn concluded<Out>(self, into: impl FnOnce(Coordinator<'shared, 'tree>, Vec<Reader<'tree>>) -> Out) -> Out {
        into(self.coordinator, self.readers)
    }
}

impl Reads for Coordinator<'_, '_> {
    fn read(self, files: &impl Files) -> Readout {
        let stage = self.shared.stage;
        let starts: Vec<Node<'_>> = stage
            .trees
            .iter()
            .zip(&stage.staged)
            .flat_map(|(tree, staged)| staged.iter().flat_map(|entry| entry.starts(tree)))
            .collect();
        let offers = starts.into_iter().enumerate().map(|(index, node)| {
            Offer::new(Key::rooted(Rank::from(index), AsRef::<Path>::as_ref(node.located())), node, None)
        });
        let current = Current { coordinator: &self, files };
        let first = current.offered(Ebb::from(&Tide::default()), offers.collect());
        let explored = Flow { current: &current }.flowed(first);
        let repo_files: Vec<RepoFile> =
            stage.staged.iter().flatten().flat_map(|entry| entry.repo_files()).cloned().collect();
        explored.concluded(|outcomes, placed, kept, survey| {
            let planned = Planner::new(&outcomes, &survey).planned(stage);
            let taken = planned.into_iter().scan(outcomes, |owned, found| {
                let outcome = owned.remove(found.as_ref());
                Some(found.taken(outcome, &placed))
            });
            <Assembled<'_> as Monoid>::concat(taken).concluded(|mains, failures| {
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
                heads.extend(assembling.into_iter().flat_map(|waiting| waiting.heads()));
                heads.sort_by(|earlier, later| AsRef::<Path>::as_ref(earlier).cmp(AsRef::<Path>::as_ref(later)));
                self.shared.table.get_or_init(|| heads);
                readout
            })
        })
    }
}

#[derive(Debug)]
struct Current<'run, 'shared, 'tree, Port> {
    coordinator: &'run Coordinator<'shared, 'tree>,
    files: &'run Port,
}

#[derive(Debug)]
struct Sending<'tree> {
    hand: Partition,
    sent: Result<(), Vec<Node<'tree>>>,
}

#[derive(Debug)]
struct Dealt<'tree> {
    offer: Offer<'tree>,
    hand: Partition,
    weight: Size,
}

impl<'tree, Port: Files> Current<'_, '_, 'tree, Port> {
    fn parsed(&self, ebb: Ebb<'_, 'tree>) -> Tide<'tree> {
        let nodes = ebb.waiting();
        let located: Vec<Located<'tree>> = nodes.iter().map(|node| node.located().clone()).collect();
        let packages = &self.coordinator.shared.stage.packages;
        Parsed::read(self.files, packages, nodes, |report, kept| {
            let landing = Tide::from(Surge::landing(located, kept, Flights::default()));
            landing.combine(self.applied(ebb, Partition::MAIN, report))
        })
    }

    fn landed(&self, ebb: Ebb<'_, 'tree>) -> Tide<'tree> {
        self.coordinator.replies.recv().map_or_else(
            |_| Tide::from(Surge::stranded()),
            |reply| {
                let landing = Tide::from(Surge::landing(Vec::new(), BTreeMap::new(), Flights::LANDED));
                reply.concluded(|hand, report| landing.combine(self.applied(ebb, hand, report)))
            },
        )
    }

    fn applied(&self, ebb: Ebb<'_, 'tree>, hand: Partition, report: Report<'tree>) -> Tide<'tree> {
        report.concluded(|sightings, facts| {
            let current = sightings.into_iter().filter_map(|sighting| {
                let claim = ebb.claim(sighting.node().located()).filter(|claim| claim.node() == sighting.node())?;
                Some(Settled::new(claim, hand, sighting.sought().map(|sought| Declared::resolved(&sought, &facts))))
            });
            let settled: Vec<Settled<'tree>> = current.collect();
            let offers: Vec<Offer<'tree>> = settled.iter().flat_map(|found| found.offers(ebb, hand, &facts)).collect();
            let offered = self.offered(ebb, offers);
            Tide::from(Surge::settling(settled, facts)).combine(offered)
        })
    }

    fn offered(&self, ebb: Ebb<'_, 'tree>, offers: Vec<Offer<'tree>>) -> Tide<'tree> {
        let mut best: BTreeMap<Located<'tree>, Offer<'tree>> = BTreeMap::new();
        offers.into_iter().filter(|offer| offer.beats(ebb.claim(offer.as_ref())).is_some()).for_each(|offer| {
            let held = best.remove(offer.as_ref());
            let kept = held.filter(|earlier| earlier.precedes(&offer).is_some()).unwrap_or(offer);
            best.insert(AsRef::<Located>::as_ref(&kept).clone(), kept);
        });
        let remotes = &self.coordinator.remotes;
        let live: Vec<Partition> = iter::once(Partition::MAIN)
            .chain(remotes.iter().map(Remote::hand))
            .filter(|hand| ebb.dead(*hand).is_none())
            .collect();
        let dealt: Vec<Dealt<'tree>> = best
            .into_values()
            .scan(BTreeMap::new(), |batch: &mut BTreeMap<Partition, Size>, offer| {
                let weight =
                    self.files.size(AsRef::<Path>::as_ref(AsRef::<Located>::as_ref(&offer))).unwrap_or_default();
                let weighed = |hand: &Partition| {
                    [ebb.load(*hand), batch.get(hand).copied().unwrap_or_default()].into_iter().sum::<Size>()
                };
                let lightest = live.iter().min_by_key(|hand| (weighed(hand), **hand)).copied();
                let hand = offer.owner().or(lightest).unwrap_or(Partition::MAIN);
                let total = [batch.get(&hand).copied().unwrap_or_default(), weight].into_iter().sum();
                batch.insert(hand, total);
                Some(Dealt { offer, hand, weight })
            })
            .collect();
        let mut by_hand: BTreeMap<Partition, Vec<Node<'tree>>> = BTreeMap::new();
        dealt.iter().for_each(|found| by_hand.entry(found.hand).or_default().push(found.offer.node().clone()));
        let local: Vec<Node<'tree>> = by_hand.remove(&Partition::MAIN).unwrap_or_default();
        let sendings: Vec<Sending<'tree>> = remotes
            .iter()
            .filter_map(|remote| {
                let nodes = by_hand.remove(&remote.hand()).unwrap_or_default();
                nodes.first()?;
                Some(Sending { hand: remote.hand(), sent: remote.dispatched(nodes) })
            })
            .collect();
        let flights = Monoid::concat(sendings.iter().filter(|sending| sending.sent.is_ok()).map(|_| Flights::SENT));
        let dead: Vec<Partition> =
            sendings.iter().filter(|sending| sending.sent.is_err()).map(|sending| sending.hand).collect();
        let returned =
            sendings.into_iter().flat_map(|sending| sending.sent.map_or_else(|nodes| nodes, |()| Vec::new()));
        let claims = dealt.into_iter().map(|found| found.offer.claimed(found.hand, found.weight)).collect();
        Tide::from(Surge::dispatched(claims, flights, local.into_iter().chain(returned).collect(), dead))
    }
}

#[derive(Debug)]
struct Flow<'current, 'run, 'shared, 'tree, Port> {
    current: &'current Current<'run, 'shared, 'tree, Port>,
}

impl<'tree, Port: Files> Flow<'_, '_, '_, 'tree, Port> {
    fn flowed(&self, tide: Tide<'tree>) -> Explored<'tree> {
        match Ebb::from(&tide).waiting().first().map(|_| ()) {
            Some(()) => self.drained(tide),
            None => self.waited(tide),
        }
    }

    fn waited(&self, tide: Tide<'tree>) -> Explored<'tree> {
        match Ebb::from(&tide).airborne() {
            Some(()) => self.landing(tide),
            None => tide.explored(),
        }
    }

    fn drained(&self, tide: Tide<'tree>) -> Explored<'tree> {
        let surge = self.current.parsed(Ebb::from(&tide));
        self.flowed(tide.combine(surge))
    }

    fn landing(&self, tide: Tide<'tree>) -> Explored<'tree> {
        let surge = self.current.landed(Ebb::from(&tide));
        self.flowed(tide.combine(surge))
    }
}

impl<'tree> Settled<'tree> {
    fn offers(&self, ebb: Ebb<'_, 'tree>, hand: Partition, facts: &lint_roots::Survey) -> Vec<Offer<'tree>> {
        let claim = self.claim(ebb);
        let declared = self.declared();
        claim
            .into_iter()
            .flat_map(|held| {
                let home = held.node().home();
                let modules = declared.into_iter().flat_map(|done| done.children(Kind::Module)).map(move |module| {
                    let node = home.child(module, facts);
                    Offer::new(
                        Key::from(Lineage::new(held.key(), AsRef::<Path>::as_ref(node.located()))),
                        node,
                        held.owned_by(hand),
                    )
                });
                let tests = declared.into_iter().flat_map(|done| done.children(Kind::Test)).map(move |test| {
                    let node = home.child(test, facts);
                    Offer::new(
                        Key::from(Lineage::new(held.key(), AsRef::<Path>::as_ref(node.located()))),
                        node,
                        Some(hand),
                    )
                });
                modules.chain(tests).collect::<Vec<_>>()
            })
            .collect()
    }
}
