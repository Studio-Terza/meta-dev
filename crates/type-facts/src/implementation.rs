use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Implementation {
    implemented: Name,
    argument: Option<Name>,
}

impl Implementation {
    pub(crate) fn new(implemented: Name, argument: Option<Name>) -> Implementation {
        Implementation { implemented, argument }
    }

    #[must_use]
    pub fn argument(&self) -> Option<&Name> {
        self.argument.as_ref()
    }
}

impl AsRef<Name> for Implementation {
    fn as_ref(&self) -> &Name {
        &self.implemented
    }
}
