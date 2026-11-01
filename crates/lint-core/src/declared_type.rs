use std::fmt;

use lint_layout::Name;

use crate::home::Home;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredType {
    home: Home,
    name: Name,
}

impl DeclaredType {
    #[must_use]
    pub fn new(home: Home, name: Name) -> DeclaredType {
        DeclaredType { home, name }
    }

    #[must_use]
    pub fn home(&self) -> &Home {
        &self.home
    }
}

impl fmt::Display for DeclaredType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(formatter)
    }
}
