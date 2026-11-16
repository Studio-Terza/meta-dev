use syntax::{Pat, PatEdge, PatReached, PatTree};

use crate::finding::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dispatch {
    Variant,
    Value,
}

impl From<&Pat> for Dispatch {
    fn from(pattern: &Pat) -> Dispatch {
        let reached = PatTree::reached(pattern, &|pat, edge| Dispatch::opens(pat, edge).is_some());
        reached.into_iter().find_map(Dispatch::valued).map_or(Dispatch::Variant, |()| Dispatch::Value)
    }
}

impl Dispatch {
    pub fn fault(self) -> Option<Finding> {
        match self {
            Dispatch::Variant => None,
            Dispatch::Value => Some(Finding::ValuePattern),
        }
    }

    fn opens(pat: &Pat, edge: PatEdge) -> Option<()> {
        match pat {
            Pat::Tuple(_) | Pat::Slice(_) => Self::nested(edge),
            Pat::Or(_) | Pat::Reference(_) | Pat::Paren(_) | Pat::Ident(_) | Pat::TupleStruct(_) | Pat::Struct(_) => {
                Some(())
            }
            _ => None,
        }
    }

    fn valued(reached: PatReached<'_>) -> Option<()> {
        match reached.pat() {
            Pat::Lit(_) | Pat::Range(_) | Pat::Const(_) => Some(()),
            Pat::Tuple(_) | Pat::Slice(_) => Self::nested(reached.edge()).map_or(Some(()), |()| None),
            _ => None,
        }
    }

    fn nested(edge: PatEdge) -> Option<()> {
        match edge {
            PatEdge::Field | PatEdge::Element => Some(()),
            PatEdge::Root | PatEdge::Alternative | PatEdge::Inner => None,
        }
    }
}
