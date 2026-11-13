use std::collections::BTreeMap;

use lint_core::{FileHead, Parsed, SourceFile};

use crate::{ItemGraph, ReferencedNames, Scope};

#[derive(Debug, Default)]
pub struct FileNames<'file> {
    named: Vec<NamedFile<'file>>,
}

#[derive(Debug)]
struct NamedFile<'file> {
    head: &'file FileHead,
    scope: Scope,
    names: ReferencedNames,
}

impl<'file> From<&'file [SourceFile]> for FileNames<'file> {
    fn from(files: &'file [SourceFile]) -> FileNames<'file> {
        files.iter().map(Parsed::from).map(FileNames::from).collect()
    }
}

impl<'head> From<Parsed<'_, 'head>> for FileNames<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> FileNames<'head> {
        let syntax = parsed.file().syntax();
        let named = parsed.head().shipped().map(|head| {
            let scope = Scope::of_file(syntax);
            NamedFile { head, names: ReferencedNames::of_items(&syntax.items, &scope), scope }
        });
        FileNames { named: named.into_iter().collect() }
    }
}

impl<'file> FromIterator<FileNames<'file>> for FileNames<'file> {
    fn from_iter<Parts: IntoIterator<Item = FileNames<'file>>>(parts: Parts) -> FileNames<'file> {
        FileNames { named: parts.into_iter().flat_map(|part| part.named).collect() }
    }
}

impl<'file> FileNames<'file> {
    pub fn each<'names, Mapped>(
        &'names self,
        mapped: impl Fn(&'file FileHead, &'names Scope, &'names ReferencedNames) -> Mapped + 'names,
    ) -> impl Iterator<Item = Mapped> + 'names {
        self.named.iter().map(move |named| mapped(named.head, &named.scope, &named.names))
    }

    #[must_use]
    pub fn graphs<Key: Ord>(&self, key: impl Fn(&'file FileHead) -> Key) -> BTreeMap<Key, ItemGraph> {
        let grouped =
            self.named.iter().fold(BTreeMap::new(), |mut crates: BTreeMap<Key, Vec<&ReferencedNames>>, named| {
                crates.entry(key(named.head)).or_default().push(&named.names);
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
