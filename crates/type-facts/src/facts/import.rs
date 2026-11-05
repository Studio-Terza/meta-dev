use lint_layout::Name;
use lint_placement::{CrateName, ModulePath};

use crate::type_fact::TypeFact;

#[derive(Clone, Debug)]
pub(crate) struct Import<'file> {
    krate: &'file CrateName,
    module: ModulePath,
    source: ModulePath,
    named: Option<Name>,
}

impl<'file> Import<'file> {
    pub(crate) fn new(
        krate: &'file CrateName,
        module: ModulePath,
        source: ModulePath,
        named: Option<Name>,
    ) -> Import<'file> {
        Import { krate, module, source, named }
    }

    pub(crate) fn made_in(&self, krate: &CrateName, module: &ModulePath) -> Option<&Import<'file>> {
        Some(self).filter(|import| import.krate == krate).filter(|import| import.module == *module)
    }

    pub(crate) fn bringing(&self, found: &TypeFact<'_>) -> Option<&Import<'file>> {
        Some(self)
            .filter(|import| AsRef::<ModulePath>::as_ref(found) == &import.source)
            .filter(|import| import.named.as_ref().is_none_or(|named| AsRef::<Name>::as_ref(found) == named))
    }
}
