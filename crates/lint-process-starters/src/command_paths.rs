use lint_core::{Site, SourceFile};
use lint_layout::{PathTail, Word, words};
use lint_paths::{Ground, Paths, Use};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    #[default]
    OutsidePorts,
    Everywhere,
}

#[derive(Debug)]
pub struct CommandPaths;

impl CommandPaths {
    const FINALS: [Word<'static>; 1] = words!["new"];
    const STARTER: [Word<'static>; 3] = words!["process", "Command", "new"];

    #[must_use]
    pub fn found(file: &SourceFile, scope: Scope) -> Vec<Site<'_>> {
        Self::among(Paths::read([file], &Self::FINALS), scope)
    }

    #[must_use]
    pub fn across(files: &[SourceFile], scope: Scope) -> Vec<Site<'_>> {
        Self::among(Paths::read(files, &Self::FINALS), scope)
    }

    fn among(uses: Vec<Use<'_>>, scope: Scope) -> Vec<Site<'_>> {
        uses.into_iter()
            .filter(|used| used.spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|used| scope == Scope::Everywhere || used.ground() == Ground::Open)
            .map(|used| used.place())
            .collect()
    }
}
