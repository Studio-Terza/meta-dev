use lint_core::{Site, SourceFile};
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
    const STARTER: [&'static str; 3] = ["process", "Command", "new"];

    #[must_use]
    pub fn found(file: &SourceFile, scope: Scope) -> Vec<Site> {
        Self::among(Paths::read([file]), scope)
    }

    #[must_use]
    pub fn across(files: &[SourceFile], scope: Scope) -> Vec<Site> {
        Self::among(Paths::read(files), scope)
    }

    fn among(uses: Vec<Use>, scope: Scope) -> Vec<Site> {
        uses.into_iter()
            .filter(|used| used.spelling(&Self::STARTER).is_some())
            .filter(|used| scope == Scope::Everywhere || used.ground() == Ground::Open)
            .map(|used| used.place().clone())
            .collect()
    }
}
