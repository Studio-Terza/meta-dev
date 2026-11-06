use lint_layout::LineNumber;

use crate::item_path::ItemPath;
use crate::resolved::Resolved;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Spelling {
    Written(ItemPath, LineNumber),
    SelfType(LineNumber),
    Implied,
}

impl Spelling {
    #[must_use]
    pub fn written(&self) -> Option<&ItemPath> {
        match self {
            Spelling::Written(path, _) => Some(path),
            Spelling::SelfType(_) | Spelling::Implied => None,
        }
    }

    #[must_use]
    pub fn line(&self) -> Option<LineNumber> {
        match self {
            Spelling::Written(_, line) | Spelling::SelfType(line) => Some(*line),
            Spelling::Implied => None,
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
    pub fn new(resolved: Resolved, spelling: Spelling) -> Sited {
        Sited { resolved, spelling }
    }

    #[must_use]
    pub fn implied(resolved: Resolved) -> Sited {
        Sited { resolved, spelling: Spelling::Implied }
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
