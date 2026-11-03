mod boundary;

use std::path::{Component, Path};

use crate::name::Name;
use crate::word::Word;
use boundary::Wildcard;

#[derive(Debug, Default)]
pub struct Members {
    listed: Vec<Name>,
    excluded: Vec<Name>,
}

impl Members {
    #[must_use]
    pub fn new(listed: Vec<Name>, excluded: Vec<Name>) -> Members {
        Members { listed, excluded }
    }

    #[must_use]
    pub fn admits<'path>(&self, relative: &'path Path) -> Option<&'path Path> {
        Self::any_matches(&self.listed, relative)
            .filter(|_| Self::any_matches(&self.excluded, relative).is_none())
            .map(|_| relative)
    }

    fn any_matches<'pattern>(patterns: &'pattern [Name], relative: &Path) -> Option<&'pattern Name> {
        patterns.iter().find(|pattern| Self::matches(pattern.as_ref(), relative).is_some())
    }

    fn matches<'path>(pattern: &Path, relative: &'path Path) -> Option<&'path Path> {
        let wanted: Vec<Component> = pattern.components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component> = relative.components().collect();
        (wanted.len() == found.len())
            .then_some(relative)
            .filter(|_| wanted.iter().zip(&found).all(|(expected, part)| Self::fits(expected, part).is_some()))
    }

    fn fits(expected: &Component, part: &Component) -> Option<()> {
        expected
            .as_os_str()
            .to_str()
            .and_then(|spelled| Name::from(part.as_os_str()).matches::<Wildcard>(&[Word::new(spelled)]))
            .map(|_| ())
    }
}
