mod cut;
mod item_count;
mod tangle;

use lint_core::{CrateFile, Least, Most, Site, Violation};

use cut::Cut;
use lint_item_graph::ItemGraph;
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

    pub fn of_graph(crate_file: &CrateFile, graph: &ItemGraph) -> CrateSummary {
        let members = graph.tangles();
        let tangles = members
            .iter()
            .filter(|members| members.len() > 1)
            .map(|members| Tangle::new(members.iter().filter_map(|member| graph.name_at(*member)).cloned().collect()))
            .collect();
        CrateSummary {
            site: Site::of_file(crate_file.as_ref()),
            tangles,
            cut: Cut::best(graph, &members, Self::CUT_SIDE),
        }
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
            .map(|cut| Violation::new(self.site.clone(), cut));
        tangles.chain(cut).collect()
    }
}
