use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use algebra::{Monoid, Semigroup};
use file_ports::Size;
use lint_core::SourceFile;
use lint_layout::Failure;
use lint_roots::Survey;

use super::assembly::Outcomes;
use crate::source_files::declared::Declared;
use crate::source_files::located::{Located, Node};
use crate::source_files::shared::boundary::{Flights, Partition, Rank};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key {
    root: Rank,
    chain: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Lineage<'parent> {
    parent: &'parent Key,
    path: &'parent Path,
}

impl<'parent> Lineage<'parent> {
    pub(super) fn new(parent: &'parent Key, path: &'parent Path) -> Lineage<'parent> {
        Lineage { parent, path }
    }
}

impl From<Lineage<'_>> for Key {
    fn from(lineage: Lineage<'_>) -> Key {
        let chain = lineage.parent.chain.iter().cloned().chain([lineage.path.to_path_buf()]).collect();
        Key { root: lineage.parent.root, chain }
    }
}

impl Key {
    pub(super) fn rooted(root: Rank, path: &Path) -> Key {
        Key { root, chain: vec![path.to_path_buf()] }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Offer<'tree> {
    key: Key,
    node: Node<'tree>,
    owner: Option<Partition>,
}

impl<'tree> Offer<'tree> {
    pub(super) fn new(key: Key, node: Node<'tree>, owner: Option<Partition>) -> Offer<'tree> {
        Offer { key, node, owner }
    }

    pub(super) fn claimed(self, hand: Partition, weight: Size) -> Claim<'tree> {
        Claim { key: self.key, node: self.node, owner: self.owner, hand, weight }
    }

    pub(super) fn owner(&self) -> Option<Partition> {
        self.owner
    }

    pub(super) fn beats(&self, held: Option<&Claim<'tree>>) -> Option<()> {
        held.map_or(Some(()), |claim| Some(()).filter(|()| self.key < claim.key))
    }

    pub(super) fn precedes(&self, other: &Offer<'tree>) -> Option<()> {
        Some(()).filter(|()| self.key <= other.key)
    }

    pub(super) fn node(&self) -> &Node<'tree> {
        &self.node
    }
}

impl<'tree> AsRef<Located<'tree>> for Offer<'tree> {
    fn as_ref(&self) -> &Located<'tree> {
        self.node.located()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Claim<'tree> {
    key: Key,
    node: Node<'tree>,
    owner: Option<Partition>,
    hand: Partition,
    weight: Size,
}

impl<'tree> Claim<'tree> {
    pub(super) fn key(&self) -> &Key {
        &self.key
    }

    pub(super) fn node(&self) -> &Node<'tree> {
        &self.node
    }

    pub(super) fn owned_by(&self, hand: Partition) -> Option<Partition> {
        self.owner.map(|_| hand)
    }
}

#[derive(Debug)]
pub(super) struct Settled<'tree> {
    key: Key,
    hand: Partition,
    outcome: Result<Declared, Failure>,
    located: Located<'tree>,
}

impl<'tree> Settled<'tree> {
    pub(super) fn new(claim: &Claim<'tree>, hand: Partition, outcome: Result<Declared, Failure>) -> Settled<'tree> {
        Settled { key: claim.key.clone(), hand, outcome, located: claim.node.located().clone() }
    }

    pub(super) fn claim<'tide>(&self, ebb: Ebb<'tide, 'tree>) -> Option<&'tide Claim<'tree>> {
        ebb.claim(&self.located)
    }

    pub(super) fn declared(&self) -> Option<&Declared> {
        self.outcome.iter().next()
    }
}

#[derive(Debug, Default)]
pub(super) struct Tide<'tree> {
    claims: BTreeMap<Located<'tree>, Claim<'tree>>,
    settled: BTreeMap<Located<'tree>, Settled<'tree>>,
    kept: BTreeMap<Located<'tree>, SourceFile>,
    survey: Survey,
    flights: Flights,
    loads: BTreeMap<Partition, Size>,
    local: BTreeMap<Located<'tree>, Node<'tree>>,
    parsed: BTreeSet<Located<'tree>>,
    dead: BTreeSet<Partition>,
    stranded: Option<()>,
}

impl Semigroup for Tide<'_> {
    fn combine(self, other: Self) -> Self {
        let mut claims = self.claims;
        other.claims.into_iter().for_each(|(located, claim)| {
            let held = claims.remove(&located);
            let kept = held.filter(|earlier| earlier.key <= claim.key).unwrap_or(claim);
            claims.insert(located, kept);
        });
        let mut settled = self.settled;
        other.settled.into_iter().for_each(|(located, found)| {
            let held = settled.remove(&located);
            let kept = held.filter(|earlier| earlier.key <= found.key).unwrap_or(found);
            settled.insert(located, kept);
        });
        let mut kept = self.kept;
        kept.extend(other.kept);
        let mut loads = self.loads;
        other.loads.into_iter().for_each(|(hand, load)| {
            let held = loads.remove(&hand).unwrap_or_default();
            loads.insert(hand, [held, load].into_iter().sum());
        });
        let mut local = self.local;
        local.extend(other.local);
        let mut parsed = self.parsed;
        parsed.extend(other.parsed);
        let mut dead = self.dead;
        dead.extend(other.dead);
        Tide {
            claims,
            settled,
            kept,
            survey: self.survey.combine(other.survey),
            flights: self.flights.combine(other.flights),
            loads,
            local,
            parsed,
            dead,
            stranded: self.stranded.or(other.stranded),
        }
    }
}

impl Monoid for Tide<'_> {}

#[derive(Debug, Default)]
pub(super) struct Surge<'tree> {
    claims: Vec<Claim<'tree>>,
    settled: Vec<Settled<'tree>>,
    kept: BTreeMap<Located<'tree>, SourceFile>,
    survey: Survey,
    flights: Flights,
    local: Vec<Node<'tree>>,
    parsed: Vec<Located<'tree>>,
    dead: Vec<Partition>,
    stranded: Option<()>,
}

impl<'tree> From<Surge<'tree>> for Tide<'tree> {
    fn from(surge: Surge<'tree>) -> Tide<'tree> {
        let loads = Monoid::concat(
            surge
                .claims
                .iter()
                .map(|claim| Tide { loads: BTreeMap::from([(claim.hand, claim.weight)]), ..Tide::default() }),
        );
        let claims =
            Monoid::concat(surge.claims.into_iter().map(|claim| Tide {
                claims: BTreeMap::from([(claim.node.located().clone(), claim)]),
                ..Tide::default()
            }));
        let settled = Monoid::concat(
            surge
                .settled
                .into_iter()
                .map(|found| Tide { settled: BTreeMap::from([(found.located.clone(), found)]), ..Tide::default() }),
        );
        Monoid::concat([
            loads,
            claims,
            settled,
            Tide {
                kept: surge.kept,
                survey: surge.survey,
                flights: surge.flights,
                local: surge.local.into_iter().map(|node| (node.located().clone(), node)).collect(),
                parsed: surge.parsed.into_iter().collect(),
                dead: surge.dead.into_iter().collect(),
                stranded: surge.stranded,
                ..Tide::default()
            },
        ])
    }
}

impl<'tree> Surge<'tree> {
    pub(super) fn dispatched(
        claims: Vec<Claim<'tree>>,
        flights: Flights,
        local: Vec<Node<'tree>>,
        dead: Vec<Partition>,
    ) -> Surge<'tree> {
        Surge { claims, flights, local, dead, ..Surge::default() }
    }

    pub(super) fn settling(settled: Vec<Settled<'tree>>, survey: Survey) -> Surge<'tree> {
        Surge { settled, survey, ..Surge::default() }
    }

    pub(super) fn landing(
        parsed: Vec<Located<'tree>>,
        kept: BTreeMap<Located<'tree>, SourceFile>,
        flights: Flights,
    ) -> Surge<'tree> {
        Surge { parsed, kept, flights, ..Surge::default() }
    }

    pub(super) fn stranded() -> Surge<'tree> {
        Surge { stranded: Some(()), ..Surge::default() }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Ebb<'tide, 'tree> {
    tide: &'tide Tide<'tree>,
}

impl<'tide, 'tree> From<&'tide Tide<'tree>> for Ebb<'tide, 'tree> {
    fn from(tide: &'tide Tide<'tree>) -> Ebb<'tide, 'tree> {
        Ebb { tide }
    }
}

impl<'tide, 'tree> Ebb<'tide, 'tree> {
    pub(super) fn claim(self, located: &Located<'tree>) -> Option<&'tide Claim<'tree>> {
        self.tide.claims.get(located)
    }

    pub(super) fn waiting(self) -> Vec<Node<'tree>> {
        self.tide
            .local
            .iter()
            .filter(|(located, _)| self.tide.parsed.get(*located).is_none())
            .map(|(_, node)| node.clone())
            .collect()
    }

    pub(super) fn load(self, hand: Partition) -> Size {
        self.tide.loads.get(&hand).copied().unwrap_or_default()
    }

    pub(super) fn dead(self, hand: Partition) -> Option<()> {
        self.tide.dead.get(&hand).map(|_| ())
    }

    pub(super) fn airborne(self) -> Option<()> {
        self.tide.stranded.map_or_else(|| self.tide.flights.airborne(), |()| None)
    }
}

impl<'tree> Tide<'tree> {
    pub(super) fn explored(self) -> Explored<'tree> {
        let hands = self.settled.iter().map(|(located, found)| (located.clone(), found.hand)).collect();
        let outcomes = self.settled.into_iter().map(|(located, found)| (located, found.outcome)).collect();
        Explored { outcomes, hands, kept: self.kept, survey: self.survey }
    }
}

#[derive(Debug)]
pub(super) struct Explored<'tree> {
    outcomes: Outcomes<'tree>,
    hands: BTreeMap<Located<'tree>, Partition>,
    kept: BTreeMap<Located<'tree>, SourceFile>,
    survey: Survey,
}

impl<'tree> Explored<'tree> {
    pub(super) fn concluded<Out>(
        self,
        into: impl FnOnce(
            Outcomes<'tree>,
            BTreeMap<Located<'tree>, Partition>,
            BTreeMap<Located<'tree>, SourceFile>,
            Survey,
        ) -> Out,
    ) -> Out {
        into(self.outcomes, self.hands, self.kept, self.survey)
    }
}
