use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Item, ItemMod};

use crate::name::Name;
use crate::role::Role;
use crate::source_file::SourceFile;

#[derive(Debug)]
pub struct BinaryFiles(BTreeSet<PathBuf>);

impl BinaryFiles {
    const MAIN_FILE: &'static str = "main.rs";
    const MODULE_FILE: &'static str = "mod.rs";
    const SOURCE_EXTENSION: &'static str = "rs";

    #[must_use]
    pub fn collect(files: &[SourceFile]) -> BinaryFiles {
        let by_path: BTreeMap<&Path, &SourceFile> = files.iter().map(|file| (file.path(), file)).collect();
        BinaryFiles(
            files
                .iter()
                .filter(|file| file.placement().role() == Role::BinaryRoot)
                .fold(BTreeSet::new(), |members, root| Self::gather(root, &by_path, members)),
        )
    }

    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.0.contains(path)
    }

    fn gather(
        file: &SourceFile,
        by_path: &BTreeMap<&Path, &SourceFile>,
        mut members: BTreeSet<PathBuf>,
    ) -> BTreeSet<PathBuf> {
        if members.insert(file.path().to_owned()) {
            Self::children(file, by_path)
                .into_iter()
                .fold(members, |members, child| Self::gather(child, by_path, members))
        } else {
            members
        }
    }

    fn children<'files>(file: &SourceFile, by_path: &BTreeMap<&Path, &'files SourceFile>) -> Vec<&'files SourceFile> {
        file.syntax()
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) => Some(module),
                _ => None,
            })
            .flat_map(|module| Self::declared_files(file.path(), module))
            .filter_map(|candidate| by_path.get(candidate.as_path()).copied())
            .collect()
    }

    fn declared_files(parent: &Path, module: &ItemMod) -> Vec<PathBuf> {
        let directory = if parent.ends_with(Self::MAIN_FILE) || parent.ends_with(Self::MODULE_FILE) {
            parent.parent().map(Path::to_path_buf)
        } else {
            Some(parent.with_extension(""))
        }
        .unwrap_or_default();
        let name = Name::from(&module.ident);
        vec![
            directory.join(&name).with_extension(Self::SOURCE_EXTENSION),
            directory.join(&name).join(Self::MODULE_FILE),
        ]
    }
}
