use crate::{NodeRef, Visit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replacement {
    old: NodeRef,
    by: NodeRef,
}

#[derive(Debug, thiserror::Error)]
#[error("{0} cannot supersede itself")]
pub struct SelfSuperseding(NodeRef);

impl Replacement {
    pub fn new(old: NodeRef, by: NodeRef) -> Result<Replacement, SelfSuperseding> {
        let refused = SelfSuperseding(old.clone());
        Some(by).filter(|by| *by != old).map(|by| Replacement { old, by }).ok_or(refused)
    }

    pub(crate) fn visited<V: Visit>(&self, visitor: V) -> V::Output {
        visitor.supersede(&self.old, &self.by)
    }
}
