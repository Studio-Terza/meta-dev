use crate::collector::Collector;
use crate::module::Module;
use crate::resolver::Resolver;
use crate::scopes::Scopes;
use lint_core::SourceFile;
use lint_layout::UsePath;

#[derive(Debug)]
pub struct Imports(Scopes);

impl Imports {
    #[must_use]
    pub fn across<'file>(files: impl IntoIterator<Item = &'file SourceFile>) -> Imports {
        let files: Vec<&SourceFile> = files.into_iter().collect();
        Imports(Collector::collect(&files))
    }

    #[must_use]
    pub fn candidates(&self, module: &Module, path: &UsePath) -> Vec<UsePath> {
        Resolver::over(&self.0).candidates(module, path)
    }
}
