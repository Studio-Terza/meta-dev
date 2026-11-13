use crate::collector::Collector;
use crate::module::Module;
use crate::resolver::Resolver;
use crate::scopes::Scopes;
use lint_core::{Parsed, SourceFile};
use lint_layout::UsePath;

#[derive(Debug)]
pub struct Imports<'file>(Scopes<'file>);

impl<'file> Imports<'file> {
    #[must_use]
    pub fn across(sources: impl IntoIterator<Item = &'file SourceFile>) -> Imports<'file> {
        let files: Vec<&SourceFile> = sources.into_iter().collect();
        Imports(Collector::collect(&files))
    }

    #[must_use]
    pub fn candidates(&self, module: &Module<'_>, path: UsePath) -> Vec<UsePath> {
        Resolver::over(&self.0).candidates(module, path)
    }
}

impl<'head> From<Parsed<'_, 'head>> for Imports<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Imports<'head> {
        Imports(Collector::collected(parsed))
    }
}

impl<'head> FromIterator<Imports<'head>> for Imports<'head> {
    fn from_iter<Parts: IntoIterator<Item = Imports<'head>>>(parts: Parts) -> Imports<'head> {
        Imports(parts.into_iter().map(|part| part.0).collect())
    }
}
