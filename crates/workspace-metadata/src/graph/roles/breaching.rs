use crate::crate_name::CrateName;
use crate::graph::DependencyKeys;

#[derive(Debug)]
pub struct Breaching<'meta> {
    dependent: &'meta CrateName,
    keys: DependencyKeys<'meta>,
}

impl<'meta> Breaching<'meta> {
    pub(super) fn new(dependent: &'meta CrateName, keys: DependencyKeys<'meta>) -> Breaching<'meta> {
        Breaching { dependent, keys }
    }

    #[must_use]
    pub fn dependent(&self) -> &'meta CrateName {
        self.dependent
    }

    #[must_use]
    pub fn keys(&self) -> &DependencyKeys<'meta> {
        &self.keys
    }
}
