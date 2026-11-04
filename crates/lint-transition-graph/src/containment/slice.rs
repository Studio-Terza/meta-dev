use std::collections::{BTreeMap, BTreeSet};

use lint_item_graph::ItemGraph;

use super::cluster::{Cluster, Part};
use super::groups::Groups;
use super::link::Link;
use super::member::Member;
use super::{Containment, Toward};

#[derive(Debug)]
pub(crate) struct Cut<'found> {
    exports: BTreeSet<Member<'found>>,
    outgoing: Vec<Link<'found>>,
}

impl<'found> Cut<'found> {
    fn around(
        cluster: Cluster<'found>,
        members: &BTreeSet<Member<'found>>,
        users: &BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    ) -> Cut<'found> {
        let outgoing: Vec<Link<'found>> = members
            .iter()
            .flat_map(|export| {
                users
                    .get(export)
                    .into_iter()
                    .flatten()
                    .filter(|user| cluster.holding(user.cluster()).is_none())
                    .map(|user| Link::new(*user, *export))
            })
            .collect();
        let exports = outgoing
            .iter()
            .map(|link| {
                let [_, export] = link.ends();
                export
            })
            .collect();
        Cut { exports, outgoing }
    }

    pub(crate) fn exports(&self) -> &BTreeSet<Member<'found>> {
        &self.exports
    }

    pub(crate) fn outgoing(&self) -> &[Link<'found>] {
        &self.outgoing
    }
}

#[derive(Debug)]
pub(crate) struct Quotient<'found> {
    parts: BTreeMap<Part<'found>, BTreeSet<Part<'found>>>,
    crossing: Vec<Link<'found>>,
}

impl<'found> Quotient<'found> {
    pub(crate) fn cycles<Found>(self, close: impl Fn(BTreeSet<Part<'found>>, &[Link<'found>]) -> Found) -> Vec<Found> {
        let graph = ItemGraph::of_references(self.parts);
        graph
            .tangles()
            .into_iter()
            .map(|tangle| {
                let parts = tangle.into_iter().filter_map(|index| graph.name_at(index).copied()).collect();
                close(parts, &self.crossing)
            })
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct Slice<'found> {
    cluster: Cluster<'found>,
    members: BTreeSet<Member<'found>>,
    cut: Cut<'found>,
}

impl<'found> Slice<'found> {
    pub(crate) fn all(
        members: impl Iterator<Item = Member<'found>>,
        users: &BTreeMap<Member<'found>, BTreeSet<Member<'found>>>,
    ) -> BTreeMap<Cluster<'found>, Slice<'found>> {
        let held =
            members.fold(BTreeMap::new(), |mut held: BTreeMap<Cluster<'found>, BTreeSet<Member<'found>>>, member| {
                member.cluster().lineage().for_each(|cluster| {
                    held.entry(cluster).or_default().insert(member);
                });
                held
            });
        held.into_iter()
            .map(|(cluster, members)| {
                let cut = Cut::around(cluster, &members, users);
                (cluster, Slice { cluster, members, cut })
            })
            .collect()
    }

    pub(crate) fn cluster(&self) -> Cluster<'found> {
        self.cluster
    }

    pub(crate) fn members(&self) -> &BTreeSet<Member<'found>> {
        &self.members
    }

    pub(crate) fn cut(&self) -> &Cut<'found> {
        &self.cut
    }

    pub(crate) fn quotient(&self, view: &Containment<'found, '_>, toward: Toward) -> Quotient<'found> {
        let part = |member: Member<'found>| Part::toward(self.cluster, member.cluster());
        let crossing: Vec<Link<'found>> = view
            .links_within(&self.members, toward)
            .into_iter()
            .filter(|link| {
                let [from, to] = link.ends().map(part);
                from != to
            })
            .collect();
        let edges =
            self.members.iter().map(|member| (part(*member), BTreeSet::new())).chain(crossing.iter().map(|link| {
                let [from, to] = link.ends().map(part);
                (from, BTreeSet::from([to]))
            }));
        let parts =
            edges.fold(BTreeMap::new(), |mut parts: BTreeMap<Part<'found>, BTreeSet<Part<'found>>>, (from, to)| {
                parts.entry(from).or_default().extend(to);
                parts
            });
        Quotient { parts, crossing }
    }

    pub(crate) fn components(&self, view: &Containment<'found, '_>, toward: Toward) -> Option<Groups<'found>> {
        Groups::linked(&self.members, view.links_within(&self.members, toward).into_iter()).split()
    }
}
