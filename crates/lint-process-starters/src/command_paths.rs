use code_model::{Ground, Model, Placed, Region};
use lint_core::{Site, SourceFile};
use lint_layout::{PathTail, Word, words};
use lint_paths::{Ground as Spot, Paths};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    #[default]
    OutsidePorts,
    Everywhere,
}

impl Scope {
    fn admits(self, ground: Ground) -> Option<Ground> {
        match self {
            Scope::Everywhere => Some(ground),
            Scope::OutsidePorts => Some(ground).filter(|candidate| *candidate == Ground::Open),
        }
    }

    fn admits_spot(self, spot: Spot) -> Option<Spot> {
        match self {
            Scope::Everywhere => Some(spot),
            Scope::OutsidePorts => Some(spot).filter(|candidate| *candidate == Spot::Open),
        }
    }
}

#[derive(Debug)]
pub struct CommandPaths;

impl CommandPaths {
    const FINALS: [Word<'static>; 1] = words!["new"];
    const STARTER: [Word<'static>; 3] = words!["process", "Command", "new"];

    #[must_use]
    pub fn across(files: &[SourceFile], scope: Scope) -> Vec<Site<'_>> {
        Paths::read(files, &Self::FINALS)
            .into_iter()
            .filter(|used| used.spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|used| scope.admits_spot(used.ground()).is_some())
            .map(|used| used.place())
            .collect()
    }

    #[must_use]
    pub fn queried(model: &Model, scope: Scope) -> Vec<Site<'_>> {
        model
            .uses()
            .filter(|placed| placed.entity().spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|placed| scope.admits(AsRef::<Region>::as_ref(placed.entity()).ground()).is_some())
            .map(Placed::site)
            .collect()
    }
}
