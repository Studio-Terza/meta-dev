mod family;

use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::node::Node;
use crate::occurrence::Occurrence;
use body_shape::{ShapeKey, TokenCount};

pub(crate) use family::Family;

#[derive(Debug)]
pub(crate) struct Families<'file>(Vec<Family<'file>>);

impl<'file> Families<'file> {
    pub(crate) fn grouped(nodes: impl Iterator<Item = Node<'file>>) -> Families<'file> {
        let groups =
            nodes.fold(BTreeMap::<ShapeKey, (TokenCount, Vec<Occurrence<'file>>)>::new(), |mut groups, node| {
                let shape = node.shape();
                groups.entry(shape.key()).or_insert_with(|| (shape.size(), Vec::new())).1.push(node.into_occurrence());
                groups
            });
        let mut repeated: Vec<Family<'file>> = groups
            .into_values()
            .filter(|(_, copies)| copies.len() > 1)
            .map(|(size, copies)| Family::new(size, copies))
            .collect();
        repeated.sort_by_key(|family| Reverse(family.size()));
        Families(repeated)
    }

    pub(crate) fn maximal(self) -> Vec<Family<'file>> {
        self.0.into_iter().fold(Vec::new(), Self::admitted)
    }

    fn admitted(mut kept: Vec<Family<'file>>, family: Family<'file>) -> Vec<Family<'file>> {
        kept.extend(family.copies().iter().any(|copy| Self::cover(&kept, copy).is_none()).then_some(family));
        kept
    }

    fn cover<'kept>(kept: &'kept [Family<'file>], copy: &Occurrence<'file>) -> Option<&'kept Occurrence<'file>> {
        kept.iter().flat_map(Family::copies).find_map(|outer| copy.lies_within(outer))
    }
}
