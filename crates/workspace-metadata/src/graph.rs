use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use crate::crate_name::CrateName;
use crate::metadata::{Metadata, Package};
use crate::package_id::PackageId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge<'meta> {
    dependent: &'meta PackageId,
    required: &'meta PackageId,
}

#[derive(Debug, Default)]
pub struct Graph<'meta> {
    packages: BTreeMap<&'meta PackageId, &'meta Package>,
    members: BTreeSet<&'meta PackageId>,
    requires: BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>,
}

impl<'meta> FromIterator<&'meta Metadata> for Graph<'meta> {
    fn from_iter<Workspaces: IntoIterator<Item = &'meta Metadata>>(workspaces: Workspaces) -> Graph<'meta> {
        workspaces.into_iter().fold(Graph::default(), |mut graph, metadata| {
            graph
                .packages
                .extend(metadata.packages().iter().filter_map(|package| Some((package.identity()?, package))));
            graph.members.extend(metadata.members());
            metadata
                .nodes()
                .iter()
                .for_each(|node| graph.requires.entry(node.as_ref()).or_default().extend(node.dependencies()));
            graph
        })
    }
}

impl<'meta> Graph<'meta> {
    #[must_use]
    pub fn macros(&self) -> BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>> {
        self.members
            .iter()
            .filter_map(|id| {
                let required = self
                    .requires
                    .get(id)
                    .into_iter()
                    .flatten()
                    .filter(|required| self.members.contains(*required))
                    .filter_map(|required| self.packages.get(required).copied().and_then(Package::proc_macro))
                    .map(Package::name)
                    .collect();
                Some((self.packages.get(id)?.name(), required))
            })
            .collect()
    }

    #[must_use]
    pub fn bypassed(&self, owned: &BTreeSet<CrateName>, owning: &BTreeSet<CrateName>) -> BTreeSet<&'meta CrateName> {
        let through = self.named(owned);
        let outside: BTreeSet<&PackageId> = self.members.difference(&self.named(owning)).copied().collect();
        let fenced: BTreeSet<&PackageId> = through.union(&self.members).copied().collect();
        let direct: BTreeSet<&PackageId> =
            outside.iter().flat_map(|id| self.edges(id)).map(|edge| edge.required).collect();
        direct.difference(&fenced).filter_map(|candidate| self.dominated(&through, &outside, candidate)).collect()
    }

    fn dominated(
        &self,
        through: &BTreeSet<&'meta PackageId>,
        outside: &BTreeSet<&'meta PackageId>,
        candidate: &'meta PackageId,
    ) -> Option<&'meta CrateName> {
        let severed: BTreeSet<Edge<'meta>> =
            outside.iter().map(|dependent| Edge { dependent, required: candidate }).collect();
        let dominated: BTreeSet<&PackageId> =
            self.reached(&BTreeSet::new(), &severed).difference(&self.reached(through, &severed)).copied().collect();
        dominated.get(candidate).and_then(|id| self.packages.get(*id)).copied().map(Package::name)
    }

    fn named(&self, names: &BTreeSet<CrateName>) -> BTreeSet<&'meta PackageId> {
        self.packages.iter().filter(|(_, package)| names.contains(package.name())).map(|(id, _)| *id).collect()
    }

    fn edges(&self, dependent: &'meta PackageId) -> impl Iterator<Item = Edge<'meta>> + use<'_, 'meta> {
        self.requires.get(dependent).into_iter().flatten().map(move |required| Edge { dependent, required })
    }

    fn reached(
        &self,
        avoided: &BTreeSet<&'meta PackageId>,
        severed: &BTreeSet<Edge<'meta>>,
    ) -> BTreeSet<&'meta PackageId> {
        let start: BTreeSet<&PackageId> = self.members.difference(avoided).copied().collect();
        iter::successors(Some((start.clone(), start)), |(seen, frontier)| {
            let barred: BTreeSet<&PackageId> = seen.union(avoided).copied().collect();
            let edges: BTreeSet<Edge<'meta>> = frontier.iter().flat_map(|id| self.edges(id)).collect();
            let required: BTreeSet<&PackageId> = edges.difference(severed).map(|edge| edge.required).collect();
            let next: BTreeSet<&PackageId> = required.difference(&barred).copied().collect();
            let grown = seen.union(&next).copied().collect();
            Some(next).filter(|next| next.first().is_some()).map(|next| (grown, next))
        })
        .last()
        .map(|(seen, _)| seen)
        .unwrap_or_default()
    }
}
