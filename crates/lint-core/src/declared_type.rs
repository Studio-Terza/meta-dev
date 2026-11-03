use std::fmt;

use lint_layout::Name;

use crate::home::Home;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredType<'file> {
    home: Home<'file>,
    name: Name,
}

impl<'file> DeclaredType<'file> {
    #[must_use]
    pub fn new(home: Home<'file>, name: Name) -> DeclaredType<'file> {
        DeclaredType { home, name }
    }

    #[must_use]
    pub fn home(&self) -> Home<'file> {
        self.home
    }

    #[must_use]
    pub fn into_name(self) -> Name {
        self.name
    }
}

impl AsRef<Name> for DeclaredType<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl fmt::Display for DeclaredType<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(formatter)
    }
}
