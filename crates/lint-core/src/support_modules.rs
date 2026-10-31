use std::path::{Path, PathBuf};

use syn::ItemMod;
use syn::visit::Visit;

use crate::name::Name;
use crate::source_file::SourceFile;
use crate::walk::Walk;

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
        if path.file_name().is_some_and(|name| Self::ROOT_FILES.iter().any(|root| name == *root)) {
            path.parent().map(Path::to_path_buf).unwrap_or_default()
        } else {
            path.with_extension("")
        }
    }
}

impl<'ast> Visit<'ast> for SupportModules {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.names.extend(
            (module.content.is_none() && Walk::outside_tests(&module.attrs) == Walk::Skipped)
                .then(|| Name::from(&module.ident)),
        );
        syn::visit::visit_item_mod(self, module);
    }
}
