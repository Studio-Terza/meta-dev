use graph::Mutation;

use super::ground::Ground;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Port,
    Owned,
}

impl Handle {
    #[must_use]
    pub fn operating(self, mutation: Mutation, ground: Ground) -> Option<()> {
        matches!((self, mutation, ground), (Handle::Port, Mutation::ThroughParameter, Ground::Store | Ground::Boundary))
            .then_some(())
    }
}
