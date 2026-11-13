use code_model::{Ground, Model, PathUse, Placed, Region};
use lint_layout::{PathTail, Word, words};

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
            Scope::OutsidePorts => Some(ground).filter(|candidate| *candidate != Ground::Ports),
        }
    }
}

#[derive(Debug)]
pub struct CommandPaths;

impl CommandPaths {
    const STARTER: [Word<'static>; 3] = words!["process", "Command", "new"];

    #[must_use]
    pub fn queried(model: &Model, scope: Scope) -> Vec<Placed<'_, &PathUse>> {
        model
            .uses()
            .filter(|placed| placed.entity().spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|placed| scope.admits(AsRef::<Region>::as_ref(placed.entity()).ground()).is_some())
            .collect()
    }
}
