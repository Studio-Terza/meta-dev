use std::borrow::Borrow;
use std::fmt;

use syn::Ident;

use crate::crate_root::CrateRoot;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateName(Name);

impl From<&Ident> for CrateName {
    fn from(ident: &Ident) -> Self {
        Self(Name::from(ident))
    }
}

impl From<&Name> for CrateName {
    fn from(name: &Name) -> Self {
        Self(name.clone())
    }
}

impl From<&CrateRoot> for CrateName {
    fn from(root: &CrateRoot) -> Self {
        let spelled = root.package().map_or_else(
            || root.directory_name().unwrap_or_default().to_string_lossy().into_owned(),
            ToString::to_string,
        );
        let underscored: String =
            spelled.chars().map(|letter| Some(letter).filter(|letter| *letter != '-').unwrap_or('_')).collect();
        Self(Name::from(underscored.as_str()))
    }
}

impl Borrow<Name> for CrateName {
    fn borrow(&self) -> &Name {
        &self.0
    }
}

impl fmt::Display for CrateName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
