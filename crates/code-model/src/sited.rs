use lint_layout::Position;

use crate::item_path::ItemPath;
use crate::resolved::Resolved;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Spelling {
    Written(ItemPath, Position),
    Implied,
}

impl Spelling {
    #[must_use]
    pub fn written(&self) -> Option<&ItemPath> {
        match self {
            Self::Written(path, _) => Some(path),
            Self::Implied => None,
        }
    }

    #[must_use]
    pub fn position(&self) -> Option<Position> {
        match self {
            Self::Written(_, position) => Some(*position),
            Self::Implied => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sited {
    resolved: Resolved,
    spelling: Spelling,
}

impl Sited {
    #[must_use]
    pub fn new(resolved: Resolved, spelling: Spelling) -> Self {
        Self { resolved, spelling }
    }

    #[must_use]
    pub fn implied(resolved: Resolved) -> Self {
        Self { resolved, spelling: Spelling::Implied }
    }
}

impl AsRef<Resolved> for Sited {
    fn as_ref(&self) -> &Resolved {
        &self.resolved
    }
}

impl AsRef<Spelling> for Sited {
    fn as_ref(&self) -> &Spelling {
        &self.spelling
    }
}
