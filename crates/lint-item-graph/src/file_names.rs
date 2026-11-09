use std::collections::BTreeMap;

use lint_core::SourceFile;

use crate::{ItemGraph, ReferencedNames};

#[derive(Debug)]
pub struct FileNames<'file> {
    named: Vec<NamedFile<'file>>,
}

#[derive(Debug)]
struct NamedFile<'file> {
    file: &'file SourceFile,
    names: ReferencedNames,
}

impl<'file> From<&'file [SourceFile]> for FileNames<'file> {
    fn from(files: &'file [SourceFile]) -> FileNames<'file> {
        let shipped = files.iter().filter_map(SourceFile::shipped);
        FileNames { named: shipped.map(|file| NamedFile { file, names: ReferencedNames::of_file(file) }).collect() }
    }
}

impl<'file> FileNames<'file> {
    pub fn each<'names, Mapped>(
        &'names self,
        mapped: impl Fn(&'file SourceFile, &'names ReferencedNames) -> Mapped + 'names,
    ) -> impl Iterator<Item = Mapped> + 'names {
        self.named.iter().map(move |named| mapped(named.file, &named.names))
    }

    #[must_use]
    pub fn graphs<Key: Ord>(&self, key: impl Fn(&'file SourceFile) -> Key) -> BTreeMap<Key, ItemGraph> {
        let grouped =
            self.named.iter().fold(BTreeMap::new(), |mut crates: BTreeMap<Key, Vec<&ReferencedNames>>, named| {
                crates.entry(key(named.file)).or_default().push(&named.names);
                crates
            });
        grouped
            .into_iter()
            .map(|(krate, names)| {
                let merged = names.into_iter().cloned().fold(ReferencedNames::default(), ReferencedNames::merged);
                (krate, merged.into_graph())
            })
            .collect()
    }
}
