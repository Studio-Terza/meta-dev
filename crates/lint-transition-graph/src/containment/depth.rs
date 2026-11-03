use std::fmt;

use lint_core::{Fix, Headline, Violation};

use crate::count::Count;

use super::cluster::{Cluster, Level};
use super::{ClusterLint, Containment};

#[derive(Debug)]
pub struct DeepClusters;

impl ClusterLint for DeepClusters {
    const HEADLINE: Headline = Headline::new(
        "report only, the deep crates and modules: a cluster with an interface is deep when it exports fewer types and traits than it keeps private, its exports being those used from outside it",
    );

    fn violations<'file>(view: &Containment<'_, 'file>) -> Vec<Violation<'file>> {
        view.clusters()
            .into_iter()
            .filter_map(|cluster| Deep::measured(view, cluster).map(|deep| (cluster, deep)))
            .filter_map(|(cluster, deep)| {
                view.first_site(&view.members(cluster)).map(|site| Violation::new(site, &deep))
            })
            .collect()
    }
}

#[derive(Debug)]
struct Deep<'found> {
    cluster: Cluster<'found>,
    exported: Count,
    private: Count,
}

impl<'found> Deep<'found> {
    fn measured(view: &Containment<'found, '_>, cluster: Cluster<'found>) -> Option<Deep<'found>> {
        let judged = match cluster.level() {
            Level::Crate | Level::Module => Some(cluster),
            Level::Workspace => None,
        }?;
        let members = view.members(judged);
        let exports = view.exports(judged, &members);
        let exported = Count::new(exports.len());
        let private = Count::new(members.difference(&exports).count());
        Some(Deep { cluster: judged, exported, private })
            .filter(|deep| deep.exported > Count::default() && deep.exported < deep.private)
    }
}

impl fmt::Display for Deep<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is deep: {} exported, {} private", self.cluster, self.exported, self.private)
    }
}

impl lint_core::Finding for Deep<'_> {
    fn fix(&self) -> Fix {
        Fix::new("nothing to fix: keep the interface this small as it grows")
    }
}
