mod cut;
mod item_count;
mod tangle;

use lint_core::{CrateRoot, Least, Most, Site, Violation};

use crate::item_graph::ItemGraph;
use cut::Cut;
use tangle::Tangle;

#[derive(Clone, Debug)]
pub struct CrateSummary {
    site: Site,
    tangles: Vec<Tangle>,
    cut: Option<Cut>,
}

impl CrateSummary {
    const TANGLE_ITEMS: Most = Most::new(40);
    const CUT_SIDE: Least = Least::new(5);

    pub fn of_graph(root: &CrateRoot, graph: &ItemGraph) -> CrateSummary {
        let members = graph.tangles();
        let tangles = members
            .iter()
            .filter(|members| members.len() > 1)
            .map(|members| Tangle::new(members.iter().filter_map(|member| graph.name_at(*member)).cloned().collect()))
            .collect();
        CrateSummary { site: Site::of_file(root.directory()), tangles, cut: Cut::best(graph, &members) }
    }

    pub fn violations(&self) -> Vec<Violation> {
        let tangles = self
            .tangles
            .iter()
            .filter(|tangle| !Self::TANGLE_ITEMS.allows(tangle.items().count()))
            .map(|tangle| Violation::new(self.site.clone(), tangle));
        let cut = self
            .cut
            .iter()
            .filter(|cut| cut.conductance().is_loose())
            .filter(|cut| Self::CUT_SIDE.allows(cut.smaller_side().count()))
            .map(|cut| Violation::new(self.site.clone(), cut));
        tangles.chain(cut).collect()
    }
}
