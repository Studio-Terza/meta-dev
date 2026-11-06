use crate::trait_path::TraitPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    Std(TraitPath),
    Crate(TraitPath),
    External(TraitPath),
}

impl Resolved {
    #[must_use]
    pub fn std(&self) -> Option<&TraitPath> {
        match self {
            Resolved::Std(path) => Some(path),
            Resolved::Crate(_) | Resolved::External(_) => None,
        }
    }
}
