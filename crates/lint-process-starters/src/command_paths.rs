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
    fn admits<Grounded: PartialEq + Copy>(self, ground: Grounded, open: Grounded) -> Option<Grounded> {
        match self {
            Scope::Everywhere => Some(ground),
            Scope::OutsidePorts => Some(ground).filter(|candidate| *candidate == open),
        }
    }
}

#[derive(Debug)]
pub struct CommandPaths;

impl CommandPaths {
    const FINALS: [Word<'static>; 1] = words!["new"];
    const STARTER: [Word<'static>; 3] = words!["process", "Command", "new"];
    const MENTIONED: [Word<'static>; 1] = words!["Command"];

    #[must_use]
    pub fn across(files: &[SourceFile], scope: Scope) -> Vec<Site<'_>> {
        let mentioned = files.iter().find(|file| file.text().mentioning(&Self::MENTIONED).is_some());
        mentioned
            .map_or_default(|_| Paths::read(files, &Self::FINALS))
            .into_iter()
            .filter(|used| used.spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|used| scope.admits(used.ground(), Spot::Open).is_some())
            .map(|used| used.place())
            .collect()
    }

    #[must_use]
    pub fn queried(model: &Model, scope: Scope) -> Vec<Site<'_>> {
        model
            .uses()
            .filter(|placed| placed.entity().spelling(PathTail::new(&Self::STARTER)).is_some())
            .filter(|placed| scope.admits(AsRef::<Region>::as_ref(placed.entity()).ground(), Ground::Open).is_some())
            .map(Placed::site)
            .collect()
    }
}
