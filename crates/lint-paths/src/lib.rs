mod collector;
mod cover;
mod ground;
mod imports;
mod module;
mod resolver;
mod scopes;
mod test_file;
mod uses;
mod walker;
mod wanted;

use collector::Collector;
use lint_core::SourceFile;
use lint_layout::Word;
use resolver::Resolver;
use walker::Walker;
use wanted::Wanted;

pub use cover::Cover;
pub use ground::Ground;
pub use imports::Imports;
pub use module::Module;
pub use test_file::TestFile;
pub use uses::Use;

#[derive(Debug)]
pub struct Paths;

impl Paths {
    #[must_use]
    pub fn read<'file>(files: impl IntoIterator<Item = &'file SourceFile>, finals: &[Word<'_>]) -> Vec<Use<'file>> {
        let files: Vec<&SourceFile> = files.into_iter().collect();
        let scopes = Collector::collect(&files);
        let resolver = Resolver::over(&scopes);
        let wanted = Wanted::among(&scopes, finals);
        files.iter().flat_map(|file| Walker::walk(file, &scopes, &wanted, &resolver)).collect()
    }

    #[must_use]
    pub fn every<'file>(files: impl IntoIterator<Item = &'file SourceFile>) -> Vec<Use<'file>> {
        let files: Vec<&SourceFile> = files.into_iter().collect();
        let scopes = Collector::collect(&files);
        let resolver = Resolver::over(&scopes);
        files.iter().flat_map(|file| Walker::walk(file, &scopes, &Wanted::Every, &resolver)).collect()
    }
}
