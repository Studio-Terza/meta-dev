use std::collections::BTreeMap;

use syn::Path;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Spelling {
    Bare(Name),
    Pathed(Name),
}

impl Spelling {
    pub(crate) fn of_path(path: &Path) -> Option<Spelling> {
        let first = Name::from(&path.segments.first()?.ident);
        match path.segments.len() {
            1 => Some(Spelling::Bare(first)),
            _ => Some(Spelling::Pathed(first)),
        }
    }

    pub(crate) fn head<'spelled>(&'spelled self, imported: &'spelled BTreeMap<Name, Name>) -> Option<&'spelled Name> {
        match self {
            Spelling::Bare(head) => imported.get(head),
            Spelling::Pathed(head) => Some(head),
        }
    }
}
