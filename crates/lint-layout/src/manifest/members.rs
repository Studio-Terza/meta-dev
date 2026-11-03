mod wildcard;

use std::path::{Component, Path};

use crate::name::Name;
use crate::word::Word;
use wildcard::Wildcard;

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
        match (Self::any_matches(&self.listed, relative), Self::any_matches(&self.excluded, relative)) {
            (Some(_), None) => Some(relative),
            _ => None,
        }
    }

    fn any_matches<'pattern>(patterns: &'pattern [Name], relative: &Path) -> Option<&'pattern Name> {
        patterns.iter().find(|pattern| Self::matches(pattern.as_ref(), relative).is_some())
    }

    fn matches<'path>(pattern: &Path, relative: &'path Path) -> Option<&'path Path> {
        let wanted: Vec<Component> = pattern.components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component> = relative.components().collect();
        (wanted.len() == found.len()
            && wanted.iter().zip(&found).all(|(expected, part)| {
                expected.as_os_str().to_str().is_some_and(|spelled| {
                    Name::from(part.as_os_str()).matches::<Wildcard>(&[Word::new(spelled)]).is_some()
                })
            }))
        .then_some(relative)
    }
}
