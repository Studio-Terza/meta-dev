use std::path::PathBuf;

use syn::ItemMod;

#[derive(Debug)]
pub struct Declaration<'items> {
    module: &'items ItemMod,
    directory: PathBuf,
}

impl<'items> Declaration<'items> {
    #[must_use]
    pub fn new(module: &'items ItemMod, directory: PathBuf) -> Declaration<'items> {
        Declaration { module, directory }
    }

    #[must_use]
    pub fn module(&self) -> &'items ItemMod {
        self.module
    }

    #[must_use]
    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }
}
