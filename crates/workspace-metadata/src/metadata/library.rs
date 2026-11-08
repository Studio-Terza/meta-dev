use std::collections::BTreeMap;
use std::path::Path;

use crate::crate_name::CrateName;
use crate::metadata::package::Form;

#[derive(Debug)]
pub struct Library<'meta> {
    name: &'meta CrateName,
    root: &'meta Path,
    form: Form,
    dependencies: BTreeMap<&'meta CrateName, &'meta CrateName>,
}

impl<'meta> Library<'meta> {
    pub(crate) fn new(
        name: &'meta CrateName,
        root: &'meta Path,
        form: Form,
        dependencies: BTreeMap<&'meta CrateName, &'meta CrateName>,
    ) -> Library<'meta> {
        Library { name, root, form, dependencies }
    }

    #[must_use]
    pub fn name(&self) -> &'meta CrateName {
        self.name
    }

    #[must_use]
    pub fn root(&self) -> &'meta Path {
        self.root
    }

    #[must_use]
    pub fn form(&self) -> Form {
        self.form
    }

    #[must_use]
    pub fn dependencies(&self) -> &BTreeMap<&'meta CrateName, &'meta CrateName> {
        &self.dependencies
    }
}
