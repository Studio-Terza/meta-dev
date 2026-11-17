use std::path::{Path, PathBuf};

use syntax::ItemMod;

use lint_core::Walk;
use lint_layout::Name;
use lint_sweep::{Facts, Fold, Kind, Visit};

#[derive(Debug)]
pub(crate) struct SupportScan;

#[derive(Debug, Default)]
pub(crate) struct SupportModules {
    names: Vec<Name>,
}

impl SupportModules {
    const ROOT_FILES: [&'static str; 3] = ["mod.rs", "lib.rs", "main.rs"];
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    #[must_use]
    pub(crate) fn paths(&self, file: &Path) -> Vec<PathBuf> {
        let directory = Self::directory_of(file);
        self.names
            .iter()
            .map(Name::to_string)
            .flat_map(|name| {
                [
                    directory.join(&name).with_extension(Self::SOURCE_EXTENSION),
                    directory.join(&name).join(Self::MODULE_FILE),
                ]
            })
            .collect()
    }

    fn directory_of(path: &Path) -> PathBuf {
        path.file_name()
            .filter(|name| Self::ROOT_FILES.iter().any(|root| name == root))
            .map_or_else(|| path.with_extension(""), |_| path.parent().map_or_default(Path::to_path_buf))
    }
}

impl Fold for SupportScan {
    type Found = SupportModules;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod]
    }

    fn on_item_mod(&self, found: SupportModules, module: &ItemMod, _: &Facts<'_, ItemMod>) -> Visit<SupportModules> {
        let support = module
            .content
            .as_ref()
            .map_or(Some(module), |_| None)
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Skipped)
            .map(|support| Name::from(&support.ident));
        Visit::descending(SupportModules { names: found.names.into_iter().chain(support).collect() })
    }
}
