use std::path::{Path, PathBuf};

use syn::{Item, ItemMod};

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
    pub const fn new(module: &'items ItemMod, directory: PathBuf) -> Self {
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
                let children = file.parent().map(Path::to_path_buf).unwrap_or_default();
                vec![Child::new(file, children)]
            },
        )
    }

    #[must_use]
    pub fn within(items: &'items [Item], directory: &Path, wanted: Walk) -> Vec<Self> {
        items.iter().flat_map(|item| Self::of_item(item, directory, wanted)).collect()
    }

    fn of_item(item: &'items Item, directory: &Path, wanted: Walk) -> Vec<Self> {
        match item {
            Item::Mod(module) => Self::of_module(module, directory, wanted),
            _ => Vec::new(),
        }
    }

    fn of_module(module: &'items ItemMod, directory: &Path, wanted: Walk) -> Vec<Self> {
        let walk = Walk::outside_tests(&module.attrs);
        let inline = module
            .content
            .iter()
            .filter(|_| walk == Walk::Entered)
            .flat_map(|(_, inner)| Self::within(inner, &directory.join(Name::from(&module.ident)), wanted));
        let declared = Some(module)
            .filter(|module| module.content.is_none())
            .filter(|_| walk == wanted)
            .map(|module| Declaration::new(module, directory.to_path_buf()));
        inline.chain(declared).collect()
    }

    #[must_use]
    pub const fn module(&self) -> &'items ItemMod {
        self.module
    }

    fn own(&self) -> PathBuf {
        self.directory.join(Name::from(&self.module.ident))
    }

    fn explicit(&self, file_directory: &Path) -> Option<PathBuf> {
        Redirects::explicit(self.module).map(|path| file_directory.join(path))
    }
}
