use std::path::{Path, PathBuf};

use syn::ItemMod;

use lint_layout::Name;

use super::child::Child;
use super::redirects::Redirects;

#[derive(Debug)]
pub struct Declaration<'items> {
    module: &'items ItemMod,
    directory: PathBuf,
}

impl<'items> Declaration<'items> {
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    #[must_use]
    pub fn new(module: &'items ItemMod, directory: PathBuf) -> Declaration<'items> {
        Declaration { module, directory }
    }

    #[must_use]
    pub fn expected(&self, file_directory: &Path) -> PathBuf {
        self.explicit(file_directory).unwrap_or_else(|| self.own().with_extension(Self::SOURCE_EXTENSION))
    }

    #[must_use]
    pub fn choices(&self, file_directory: &Path) -> Vec<Child> {
        let own = self.own();
        match self.explicit(file_directory) {
            Some(file) => {
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                vec![Child::new(file, children)]
            }
            None => vec![
                Child::new(own.with_extension(Self::SOURCE_EXTENSION), own.clone()),
                Child::new(own.join(Self::MODULE_FILE), own),
            ],
        }
    }

    #[must_use]
    pub fn module(&self) -> &'items ItemMod {
        self.module
    }

    fn own(&self) -> PathBuf {
        self.directory.join(Name::from(&self.module.ident))
    }

    fn explicit(&self, file_directory: &Path) -> Option<PathBuf> {
        Redirects::explicit(self.module).map(|path| file_directory.join(path))
    }
}
