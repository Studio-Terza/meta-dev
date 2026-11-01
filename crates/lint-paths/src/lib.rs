mod collector;
mod cover;
mod ground;
mod module;
mod resolver;
mod scopes;
mod uses;
mod walker;

use collector::Collector;
use lint_core::SourceFile;
use resolver::Resolver;
use walker::Walker;

pub use cover::Cover;
pub use ground::Ground;
pub use uses::Use;

#[derive(Debug)]
pub struct Paths;

impl Paths {
    #[must_use]
    pub fn read<'file>(files: impl IntoIterator<Item = &'file SourceFile>) -> Vec<Use> {
        let files: Vec<&SourceFile> = files.into_iter().collect();
        let scopes = Collector::collect(&files);
        let resolver = Resolver::over(&scopes);
        files.iter().flat_map(|file| Walker::walk(file, &scopes, &resolver)).collect()
    }
}
