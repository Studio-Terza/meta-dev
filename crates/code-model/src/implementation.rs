use lint_layout::Name;

use crate::trait_path::TraitPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Written,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Named(Name),
    Metavariable(Name),
}

impl Subject {
    #[must_use]
    pub fn sigil(&self) -> &'static str {
        match self {
            Subject::Named(_) => "",
            Subject::Metavariable(_) => "$",
        }
    }
}

impl AsRef<Name> for Subject {
    fn as_ref(&self) -> &Name {
        match self {
            Subject::Named(name) | Subject::Metavariable(name) => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    subject: Subject,
    implemented: Option<TraitPath>,
    origin: Origin,
}

impl Implementation {
    #[must_use]
    pub fn new(subject: Subject, implemented: Option<TraitPath>, origin: Origin) -> Implementation {
        Implementation { subject, implemented, origin }
    }

    #[must_use]
    pub fn written(&self) -> Option<&Implementation> {
        match self.origin {
            Origin::Written => Some(self),
            Origin::Derived => None,
        }
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&TraitPath> {
        self.implemented.as_ref()
    }

    #[must_use]
    pub fn subject(&self) -> &Subject {
        &self.subject
    }
}
