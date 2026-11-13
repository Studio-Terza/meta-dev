mod collector;
mod cover;
mod ground;
mod resolver;
mod scopes;
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
use uses::Use;

#[derive(Debug)]
pub struct Paths;

impl Paths {
    #[must_use]
    pub fn read<'file>(sources: impl IntoIterator<Item = &'file SourceFile>, finals: &[Word<'_>]) -> Vec<Use<'file>> {
        let files: Vec<&SourceFile> = sources.into_iter().collect();
        let scopes = Collector::collect(&files);
        let resolver = Resolver::over(&scopes);
        let wanted = Wanted::among(&scopes, finals);
        files.iter().flat_map(|file| Walker::walk(file, &scopes, &wanted, &resolver)).collect()
    }

    #[must_use]
    pub fn every<'file>(sources: impl IntoIterator<Item = &'file SourceFile>) -> Vec<Use<'file>> {
        let files: Vec<&SourceFile> = sources.into_iter().collect();
        let scopes = Collector::collect(&files);
        let resolver = Resolver::over(&scopes);
        files.iter().flat_map(|file| Walker::walk(file, &scopes, &Wanted::Every, &resolver)).collect()
    }
}
