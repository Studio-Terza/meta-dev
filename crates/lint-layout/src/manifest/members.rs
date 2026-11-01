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
    pub fn admits(&self, relative: &Path) -> bool {
        Self::any_matches(&self.listed, relative) && !Self::any_matches(&self.excluded, relative)
    }

    fn any_matches(patterns: &[Name], relative: &Path) -> bool {
        patterns.iter().any(|pattern| Self::matches(pattern.as_ref(), relative))
    }

    fn matches(pattern: &Path, relative: &Path) -> bool {
        let wanted: Vec<Component> = pattern.components().filter(|part| matches!(part, Component::Normal(_))).collect();
        let found: Vec<Component> = relative.components().collect();
        wanted.len() == found.len()
            && wanted.iter().zip(&found).all(|(expected, part)| {
                expected
                    .as_os_str()
                    .to_str()
                    .is_some_and(|spelled| Name::from(part.as_os_str()).matches::<Wildcard>(&[Word::new(spelled)]))
            })
    }
}
