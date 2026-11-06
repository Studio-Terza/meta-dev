use std::fmt;

use syn::Path;

use crate::name::Name;
use crate::path_tail::PathTail;
use crate::word::Word;
use newtype::Newtype;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsePath(Vec<Name>);

impl UsePath {
    const SEPARATOR: &'static str = "::";

    pub(crate) fn push(&mut self, segment: Name) {
        self.0.push(segment);
    }

    pub(crate) fn pop(&mut self) {
        self.0.pop();
    }

    #[must_use]
    pub fn joined(&self, rest: &[Name]) -> UsePath {
        UsePath(self.0.iter().chain(rest).cloned().collect())
    }

    #[must_use]
    pub fn rerooted(&self, root: &UsePath) -> UsePath {
        UsePath(root.0.iter().chain(self.0.iter().skip(1)).cloned().collect())
    }

    #[must_use]
    pub fn rest(&self) -> UsePath {
        UsePath(self.0.iter().skip(1).cloned().collect())
    }

    #[must_use]
    pub fn ends_with(&self, tail: PathTail<'_>) -> Option<&UsePath> {
        let words = tail.into_inner();
        (self.0.len() >= words.len()).then_some(self).filter(|path| {
            path.0.iter().rev().zip(words.iter().rev()).all(|(segment, spelled)| *segment == Name::from(*spelled))
        })
    }

    #[must_use]
    pub fn spells(&self, segments: &[Word<'_>]) -> Option<&UsePath> {
        (self.0.len() == segments.len())
            .then_some(self)
            .filter(|path| path.0.iter().zip(segments).all(|(segment, spelled)| *segment == Name::from(*spelled)))
    }

    #[must_use]
    pub fn segments(&self) -> &[Name] {
        &self.0
    }

    #[must_use]
    pub fn naming<'path>(&'path self, typed: impl Fn(&'path Name) -> Option<&'path Name>) -> UsePath {
        let named = self.0.iter().position(|segment| typed(segment).is_some()).map_or(self.0.len(), |at| at + 1);
        UsePath(self.0.iter().take(named).cloned().collect())
    }
}

impl FromIterator<Name> for UsePath {
    fn from_iter<Segments: IntoIterator<Item = Name>>(segments: Segments) -> UsePath {
        UsePath(segments.into_iter().collect())
    }
}

impl From<&Path> for UsePath {
    fn from(path: &Path) -> UsePath {
        UsePath(path.segments.iter().map(|segment| Name::from(&segment.ident)).collect())
    }
}

impl fmt::Display for UsePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(Name::to_string).collect();
        formatter.write_str(&spelled.join(Self::SEPARATOR))
    }
}
