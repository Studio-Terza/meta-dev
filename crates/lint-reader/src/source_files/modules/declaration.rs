use std::path::{Path, PathBuf};

use syntax::{Item, ItemMod, Nested};

use lint_core::Walk;

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
        self.explicit(file_directory).map_or_else(
            || {
                let own = self.own();
                vec![
                    Child::new(own.with_extension(Self::SOURCE_EXTENSION), own.clone()),
                    Child::new(own.join(Self::MODULE_FILE), own),
                ]
            },
            |file| {
                let children = file.parent().map_or_default(Path::to_path_buf);
                vec![Child::new(file, children)]
            },
        )
    }

    #[must_use]
    pub fn within(items: &'items [Item], directory: &Path, wanted: Walk) -> Vec<Declaration<'items>> {
        Nested::modules(items)
            .into_iter()
            .filter(|nested| nested.ancestors().iter().all(|outer| Walk::outside_tests(&outer.attrs) == Walk::Entered))
            .filter(|nested| nested.module().content.is_none())
            .filter(|nested| Walk::outside_tests(&nested.module().attrs) == wanted)
            .map(|nested| {
                let joined = nested
                    .ancestors()
                    .iter()
                    .fold(directory.to_path_buf(), |path, outer| path.join(Name::from(&outer.ident)));
                Declaration::new(nested.module(), joined)
            })
            .collect()
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
