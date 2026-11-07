use lint_layout::Name;

use crate::implementation::Implementation;

#[derive(Clone, Copy, Debug)]
pub struct Outside<'found, 'file> {
    name: &'found Name,
    implementation: &'found Implementation<'file>,
}

impl<'found, 'file> Outside<'found, 'file> {
    pub(crate) fn new(name: &'found Name, implementation: &'found Implementation<'file>) -> Self {
        Outside { name, implementation }
    }

    #[must_use]
    pub fn name(&self) -> &'found Name {
        self.name
    }

    #[must_use]
    pub fn implementation(&self) -> &'found Implementation<'file> {
        self.implementation
    }
}
