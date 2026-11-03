use crate::collector::Collector;
use crate::module::Module;
use crate::resolver::Resolver;
use crate::scopes::Scopes;
use lint_core::SourceFile;
use lint_layout::UsePath;

#[derive(Debug)]
pub struct Imports<'file>(Scopes<'file>);

impl<'file> Imports<'file> {
    #[must_use]
    pub fn across(files: impl IntoIterator<Item = &'file SourceFile>) -> Imports<'file> {
        let files: Vec<&SourceFile> = files.into_iter().collect();
        Imports(Collector::collect(&files))
    }

    #[must_use]
    pub fn candidates(&self, module: &Module<'_>, path: UsePath) -> Vec<UsePath> {
        Resolver::over(&self.0).candidates(module, path)
    }
}
