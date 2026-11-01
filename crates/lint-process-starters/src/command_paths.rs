use lint_core::{Site, SourceFile};
use lint_layout::{Word, words};
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
    pub fn found(file: &SourceFile, scope: Scope) -> Vec<Site> {
        Self::among(Paths::read([file], &Self::FINALS), scope)
    }

    #[must_use]
    pub fn across(files: &[SourceFile], scope: Scope) -> Vec<Site> {
        Self::among(Paths::read(files, &Self::FINALS), scope)
    }

    fn among(uses: Vec<Use>, scope: Scope) -> Vec<Site> {
        uses.into_iter()
            .filter(|used| used.spelling(&Self::STARTER).is_some())
            .filter(|used| scope == Scope::Everywhere || used.ground() == Ground::Open)
            .map(|used| used.place().clone())
            .collect()
    }
}
