use std::collections::BTreeSet;

use crate::crate_name::CrateName;

#[derive(Debug, Default)]
pub struct DependencyKeys<'meta>(BTreeSet<&'meta CrateName>);

impl<'meta> FromIterator<&'meta CrateName> for DependencyKeys<'meta> {
    fn from_iter<Keys: IntoIterator<Item = &'meta CrateName>>(keys: Keys) -> DependencyKeys<'meta> {
        DependencyKeys(keys.into_iter().collect())
    }
}

impl<'meta> Extend<&'meta CrateName> for DependencyKeys<'meta> {
    fn extend<Keys: IntoIterator<Item = &'meta CrateName>>(&mut self, keys: Keys) {
        self.0.extend(keys);
    }
}

impl<'meta> AsRef<BTreeSet<&'meta CrateName>> for DependencyKeys<'meta> {
    fn as_ref(&self) -> &BTreeSet<&'meta CrateName> {
        &self.0
    }
}
