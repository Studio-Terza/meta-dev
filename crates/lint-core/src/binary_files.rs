use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::ItemMod;

use crate::syntax::Syntax;

use crate::source_file::SourceFile;
use lint_layout::Name;
use lint_placement::Role;

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
    pub fn holding<'path>(&self, path: &'path Path) -> Option<&'path Path> {
        self.0.contains(path).then_some(path)
    }

    fn gather(
        file: &SourceFile,
        by_path: &BTreeMap<&Path, &SourceFile>,
        members: BTreeSet<PathBuf>,
    ) -> BTreeSet<PathBuf> {
        match members.get(file.path()).map(|_| ()) {
            Some(()) => members,
            None => Self::children(file, by_path)
                .into_iter()
                .fold(Self::joined_by(members, file.path()), |gathered, child| Self::gather(child, by_path, gathered)),
        }
    }

    fn joined_by(mut members: BTreeSet<PathBuf>, path: &Path) -> BTreeSet<PathBuf> {
        members.insert(path.to_owned());
        members
    }

    fn children<'files>(file: &SourceFile, by_path: &BTreeMap<&Path, &'files SourceFile>) -> Vec<&'files SourceFile> {
        file.syntax()
            .items
            .iter()
            .filter_map(Syntax::item_mod)
            .flat_map(|module| Self::declared_files(file.path(), module))
            .filter_map(|candidate| by_path.get(candidate.as_path()).copied())
            .collect()
    }

    fn declared_files(parent: &Path, module: &ItemMod) -> Vec<PathBuf> {
        let directory = [Self::MAIN_FILE, Self::MODULE_FILE]
            .iter()
            .find(|root| parent.ends_with(root))
            .map_or_else(|| Some(parent.with_extension("")), |_| parent.parent().map(Path::to_path_buf))
            .unwrap_or_default();
        let name = Name::from(&module.ident);
        vec![
            directory.join(&name).with_extension(Self::SOURCE_EXTENSION),
            directory.join(&name).join(Self::MODULE_FILE),
        ]
    }
}
