use std::fmt;

use syntax::Path;

use crate::name::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsePath(Vec<Name>);

impl UsePath {
    const SEPARATOR: &'static str = "::";
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

impl AsRef<[Name]> for UsePath {
    fn as_ref(&self) -> &[Name] {
        &self.0
    }
}
