use std::collections::BTreeSet;
use std::iter;

use crate::package_id::PackageId;

use super::Graph;

#[derive(Debug, Default)]
pub(super) struct Served<'meta>(BTreeSet<&'meta PackageId>);

impl<'meta> FromIterator<&'meta PackageId> for Served<'meta> {
    fn from_iter<Ids: IntoIterator<Item = &'meta PackageId>>(ids: Ids) -> Served<'meta> {
        Served(ids.into_iter().collect())
    }
}

impl<'meta> Served<'meta> {
    pub(super) fn closed(self, graph: &Graph<'meta>) -> Closed<'meta> {
        let closure = iter::successors(Some(self), |found| {
            let joining = found.joining(graph);
            joining.first().map(|_| Served(found.0.union(&joining).copied().collect()))
        })
        .last()
        .unwrap_or_default();
        Closed(closure.0)
    }

    fn joining(&self, graph: &Graph<'meta>) -> BTreeSet<&'meta PackageId> {
        graph.members.difference(&self.0).copied().filter(|id| self.served_by(graph, id).is_some()).collect()
    }

    fn served_by(&self, graph: &Graph<'meta>, required: &'meta PackageId) -> Option<BTreeSet<&'meta PackageId>> {
        let dependents: BTreeSet<&PackageId> = graph
            .members
            .iter()
            .copied()
            .filter(|dependent| graph.requires.direct(dependent).any(|candidate| candidate == required))
            .collect();
        Some(dependents).filter(|found| found.first().is_some()).filter(|found| found.is_subset(&self.0))
    }
}

#[derive(Debug)]
pub(super) struct Closed<'meta>(BTreeSet<&'meta PackageId>);

impl<'meta> Closed<'meta> {
    pub(super) fn iter(&self) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.iter().copied()
    }
}
