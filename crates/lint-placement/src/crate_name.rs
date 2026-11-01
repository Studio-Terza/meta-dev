use std::fmt;

use syn::Ident;

use crate::crate_root::CrateRoot;
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateName(Name);

impl From<&Ident> for CrateName {
    fn from(ident: &Ident) -> CrateName {
        CrateName(Name::from(ident))
    }
}

impl From<&CrateRoot> for CrateName {
    fn from(root: &CrateRoot) -> CrateName {
        let spelled = root.package().map_or_else(
            || root.directory_name().unwrap_or_default().to_string_lossy().into_owned(),
            ToString::to_string,
        );
        let underscored: String = spelled.chars().map(|letter| if letter == '-' { '_' } else { letter }).collect();
        CrateName(Name::from(underscored.as_str()))
    }
}

impl fmt::Display for CrateName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
