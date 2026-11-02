mod family;

use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::node::Node;
use crate::occurrence::Occurrence;
use body_shape::{ShapeKey, TokenCount};

pub(crate) use family::Family;

#[derive(Debug)]
pub(crate) struct Families(Vec<Family>);

impl Families {
    pub(crate) fn grouped(nodes: impl Iterator<Item = Node>) -> Families {
        let groups = nodes.fold(BTreeMap::<ShapeKey, (TokenCount, Vec<Occurrence>)>::new(), |mut groups, node| {
            let shape = node.shape();
            groups.entry(shape.key()).or_insert_with(|| (shape.size(), Vec::new())).1.push(node.into_occurrence());
            groups
        });
        let mut repeated: Vec<Family> = groups
            .into_values()
            .filter(|(_, copies)| copies.len() > 1)
            .map(|(size, copies)| Family::new(size, copies))
            .collect();
        repeated.sort_by_key(|family| Reverse(family.size()));
        Families(repeated)
    }

    pub(crate) fn maximal(self) -> Vec<Family> {
        self.0.into_iter().fold(Vec::new(), Self::admitted)
    }

    fn admitted(mut kept: Vec<Family>, family: Family) -> Vec<Family> {
        kept.extend(family.copies().iter().any(|copy| !Self::is_covered(&kept, copy)).then_some(family));
        kept
    }

    fn is_covered(kept: &[Family], copy: &Occurrence) -> bool {
        kept.iter().flat_map(Family::copies).any(|outer| copy.lies_within(outer))
    }
}
