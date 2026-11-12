use std::path::{Path, PathBuf};

use syn::ItemMod;
use syn::visit::Visit;

use crate::source_file::SourceFile;
use crate::walk::Walk;
use lint_layout::Name;

#[derive(Debug, Default)]
pub struct SupportModules {
    names: Vec<Name>,
}

impl SupportModules {
    const ROOT_FILES: [&'static str; 3] = ["mod.rs", "lib.rs", "main.rs"];
    const SOURCE_EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    #[must_use]
    pub fn paths(file: &SourceFile) -> Vec<PathBuf> {
        let mut modules = SupportModules::default();
        modules.visit_file(file.syntax());
        let directory = Self::directory_of(file.path());
        modules
            .names
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

impl<'ast> Visit<'ast> for SupportModules {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.names.extend(
            module
                .content
                .as_ref()
                .map_or(Some(module), |_| None)
                .filter(|candidate| Walk::outside_tests(&candidate.attrs) == Walk::Skipped)
                .map(|support| Name::from(&support.ident)),
        );
        syn::visit::visit_item_mod(self, module);
    }
}
