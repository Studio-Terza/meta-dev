use lint_core::{FileHead, Parsed, SourceFile};

use crate::Scope;

#[derive(Debug, Default)]
pub struct FileScopes<'file> {
    scoped: Vec<ScopedFile<'file>>,
}

#[derive(Debug)]
struct ScopedFile<'file> {
    head: &'file FileHead,
    scope: Scope,
}

impl<'file> From<&'file [SourceFile]> for FileScopes<'file> {
    fn from(files: &'file [SourceFile]) -> FileScopes<'file> {
        files.iter().map(Parsed::from).map(FileScopes::from).collect()
    }
}

impl<'head> From<Parsed<'_, 'head>> for FileScopes<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> FileScopes<'head> {
        let scoped =
            parsed.head().shipped().map(|head| ScopedFile { head, scope: Scope::of_file(parsed.file().syntax()) });
        FileScopes { scoped: scoped.into_iter().collect() }
    }
}

impl<'file> FromIterator<FileScopes<'file>> for FileScopes<'file> {
    fn from_iter<Parts: IntoIterator<Item = FileScopes<'file>>>(parts: Parts) -> FileScopes<'file> {
        FileScopes { scoped: parts.into_iter().flat_map(|part| part.scoped).collect() }
    }
}

impl<'file> FileScopes<'file> {
    pub fn each<'scopes, Mapped>(
        &'scopes self,
        mapped: impl Fn(&'file FileHead, &'scopes Scope) -> Mapped + 'scopes,
    ) -> impl Iterator<Item = Mapped> + 'scopes {
        self.scoped.iter().map(move |scoped| mapped(scoped.head, &scoped.scope))
    }
}
