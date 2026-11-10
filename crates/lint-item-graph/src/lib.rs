mod file_names;
mod node_index;
mod referenced_names;
mod scope;
mod traversal;

pub use file_names::FileNames;
pub use referenced_names::ReferencedNames;
pub use scope::Scope;
pub use traversal::Traversal;

use node_index::NodeIndex;

use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;

#[derive(Clone, Debug, Default)]
pub struct ItemGraph<Node = Name> {
    names: Vec<Node>,
    links: Vec<Vec<NodeIndex>>,
    stand_ins: BTreeSet<NodeIndex>,
}

impl ItemGraph {
    fn standing_in(self, names: &BTreeSet<Name>) -> ItemGraph {
        let indexed = self.names.iter().enumerate().filter(|(_, name)| names.contains(*name));
        ItemGraph { stand_ins: indexed.map(|(index, _)| NodeIndex::from(index)).collect(), ..self }
    }

    #[must_use]
    pub fn stand_ins(&self) -> BTreeMap<&Name, Vec<&Name>> {
        self.stand_ins
            .iter()
            .filter_map(|index| self.name_at(*index).map(|name| (name, self.used_by(*index))))
            .collect()
    }

    fn used_by(&self, index: NodeIndex) -> Vec<&Name> {
        self.links.get(index.into_inner()).into_iter().flatten().filter_map(|used| self.name_at(*used)).collect()
    }
}

impl<Node: Ord> ItemGraph<Node> {
    #[must_use]
    pub fn of_references<Uses>(references: BTreeMap<Node, Uses>) -> ItemGraph<Node>
    where
        for<'uses> &'uses Uses: IntoIterator<Item = &'uses Node>,
    {
        let index_of: BTreeMap<&Node, NodeIndex> =
            references.keys().enumerate().map(|(index, name)| (name, NodeIndex::from(index))).collect();
        let links = references
            .iter()
            .map(|(owner, uses)| {
                let linked: BTreeSet<NodeIndex> = uses
                    .into_iter()
                    .filter(|name| *name != owner)
                    .filter_map(|name| index_of.get(name).copied())
                    .collect();
                linked.into_iter().collect()
            })
            .collect();
        ItemGraph { names: references.into_keys().collect(), links, stand_ins: BTreeSet::new() }
    }

    fn name_at(&self, node: NodeIndex) -> Option<&Node> {
        self.names.get(node.into_inner())
    }

    #[must_use]
    pub fn tangles(&self) -> Vec<Vec<&Node>> {
        let everyone = (0..self.names.len()).map(NodeIndex::from);
        let forward = |node: NodeIndex| self.links.get(node.into_inner()).into_iter().flatten().copied();
        let finished = Traversal::trees(everyone, forward).into_iter().flatten().rev();
        let backward = Self::reversed(&self.links);
        let distinct: BTreeSet<Vec<NodeIndex>> = Traversal::trees(finished, |node: NodeIndex| {
            backward.get(node.into_inner()).into_iter().flatten().copied()
        })
        .into_iter()
        .map(|component| component.into_iter().collect::<BTreeSet<NodeIndex>>().into_iter().collect())
        .collect();
        let mut tangles: Vec<Vec<NodeIndex>> = distinct.into_iter().collect();
        tangles.sort_by_key(|tangle| std::cmp::Reverse(tangle.len()));
        tangles.into_iter().map(|tangle| tangle.into_iter().filter_map(|index| self.name_at(index)).collect()).collect()
    }

    fn reversed(links: &[Vec<NodeIndex>]) -> Vec<Vec<NodeIndex>> {
        let edges = links
            .iter()
            .enumerate()
            .flat_map(|(from, targets)| targets.iter().map(move |to| (NodeIndex::from(from), *to)));
        edges.fold(vec![Vec::new(); links.len()], |mut backward, (from, to)| {
            backward.get_mut(to.into_inner()).into_iter().for_each(|reaching| reaching.push(from));
            backward
        })
    }
}
