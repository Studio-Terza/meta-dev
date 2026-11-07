use std::collections::BTreeMap;

use syn::{Path, PathSegment};

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Spelling {
    Bare(Name),
    Pathed(Name),
}

impl Spelling {
    pub(crate) fn of_path(path: &Path) -> Option<Self> {
        path.segments.first().map(|first| Self::of_head(Name::from(&first.ident), path.segments.iter().nth(1)))
    }

    const fn of_head(head: Name, second: Option<&PathSegment>) -> Self {
        match second {
            Some(_) => Self::Pathed(head),
            None => Self::Bare(head),
        }
    }

    pub(crate) fn head<'spelled>(&'spelled self, imported: &'spelled BTreeMap<Name, Name>) -> Option<&'spelled Name> {
        match self {
            Self::Bare(head) => imported.get(head),
            Self::Pathed(head) => Some(head),
        }
    }
}
