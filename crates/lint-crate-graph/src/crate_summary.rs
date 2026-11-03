mod item_count;
mod tangle;

use lint_core::{Most, Site, Violation};
use lint_placement::CrateFile;

use lint_item_graph::ItemGraph;
use tangle::Tangle;

#[derive(Clone, Debug)]
pub struct CrateSummary<'file> {
    site: Site<'file>,
    tangles: Vec<Tangle>,
}

impl<'file> CrateSummary<'file> {
    const TANGLE_ITEMS: Most = Most::new(40);

    pub fn of_graph(crate_file: &'file CrateFile, graph: &ItemGraph) -> CrateSummary<'file> {
        let members = graph.tangles();
        let tangles = members
            .iter()
            .map(|members| Tangle::new(members.iter().filter_map(|member| graph.name_at(*member)).cloned().collect()))
            .collect();
        CrateSummary { site: Site::of_file(crate_file.as_ref()), tangles }
    }

    pub fn violations(&self) -> Vec<Violation<'file>> {
        self.tangles
            .iter()
            .filter(|tangle| Self::TANGLE_ITEMS.allows(tangle.items()).is_none())
            .map(|tangle| Violation::new(self.site, tangle))
            .collect()
    }
}
