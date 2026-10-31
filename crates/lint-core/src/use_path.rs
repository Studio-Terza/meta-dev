use std::fmt;

use syn::Path;

use crate::name::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsePath(Vec<Name>);

impl UsePath {
    const SEPARATOR: &'static str = "::";

    #[must_use]
    pub fn extended(&self, segment: Name) -> UsePath {
        UsePath(self.0.iter().cloned().chain(Some(segment)).collect())
    }

    #[must_use]
    pub fn rerooted(&self, root: &UsePath) -> UsePath {
        UsePath(root.0.iter().chain(self.0.iter().skip(1)).cloned().collect())
    }

    #[must_use]
    pub fn first(&self) -> Option<&Name> {
        self.0.first()
    }

    #[must_use]
    pub fn spells(&self, segments: &[&str]) -> bool {
        self.0.len() == segments.len() && self.0.iter().zip(segments).all(|(segment, spelled)| segment == spelled)
    }

    #[must_use]
    pub fn segments(&self) -> &[Name] {
        &self.0
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
