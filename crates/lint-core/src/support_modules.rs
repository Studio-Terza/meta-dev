use std::path::{Path, PathBuf};

use syntax::ItemMod;

use crate::contribution::Contributed;
use crate::joins::Joins;
use crate::walk::Walk;
use crate::watch::{Contribute, Facts, Kind};
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct SupportModules {
    names: Vec<Name>,
}

#[derive(Debug)]
pub struct SupportScan;

impl SupportModules {
    const ROOT_FILES: [&'static str; 3] = ["mod.rs", "lib.rs", "main.rs"];
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    #[must_use]
    pub fn paths(&self, file: &Path) -> Vec<PathBuf> {
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

impl Joins for SupportModules {
    fn combine(self, other: SupportModules) -> SupportModules {
        SupportModules { names: self.names.combine(other.names) }
    }
}

impl Contribute for SupportScan {
    type Open = ();
    type Part = SupportModules;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod]
    }

    fn on_item_mod(
        &self,
        module: &ItemMod,
        _: &Facts<'_, ItemMod>,
        _around: &crate::Around<'_, Self::Open>,
    ) -> Contributed<(), SupportModules> {
        let names = module
            .content
            .as_ref()
            .map_or(Some(module), |_| None)
            .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Skipped)
            .map(|support| Name::from(&support.ident))
            .into_iter()
            .collect();
        Contributed::parted(SupportModules { names })
    }
}
