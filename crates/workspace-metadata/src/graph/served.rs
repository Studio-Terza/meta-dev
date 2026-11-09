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
    pub(super) fn closed(self, graph: &Graph<'meta>) -> Served<'meta> {
        iter::successors(Some(self), |found| {
            let joining = found.joining(graph);
            joining.first().map(|_| Served(found.0.union(&joining).copied().collect()))
        })
        .last()
        .unwrap_or_default()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &'meta PackageId> + use<'_, 'meta> {
        self.0.iter().copied()
    }

    fn joining(&self, graph: &Graph<'meta>) -> BTreeSet<&'meta PackageId> {
        graph
            .members
            .iter()
            .copied()
            .filter(|id| self.0.get(id).is_none())
            .filter_map(|id| self.serving(graph, id))
            .collect()
    }

    fn serving(&self, graph: &Graph<'meta>, required: &'meta PackageId) -> Option<&'meta PackageId> {
        let dependents: BTreeSet<&PackageId> = graph
            .members
            .iter()
            .copied()
            .filter(|dependent| graph.requires.direct(dependent).any(|candidate| candidate == required))
            .collect();
        dependents.first().filter(|_| dependents.is_subset(&self.0)).map(|_| required)
    }
}
