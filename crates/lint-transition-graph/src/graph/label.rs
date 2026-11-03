use std::fmt;

use lint_layout::Name;
use transition_facts::Transition;

use crate::edge::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Label<'found> {
    Written,
    Trait(&'found Name),
    Lift,
    Operation(&'found Name),
}

impl<'found> Label<'found> {
    pub(crate) fn read(role: Role, transition: &Transition<'found, '_>) -> Label<'found> {
        match role {
            Role::Operation => Label::Operation(transition.function().as_ref()),
            Role::Lift => Label::Lift,
            Role::Written => transition.context().trait_name().map_or(Label::Written, Label::Trait),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Lifted;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Interface<'found>(&'found Name);

impl<'found> Label<'found> {
    pub(crate) fn written(&self) -> Option<&Label<'found>> {
        match self {
            Label::Written | Label::Trait(_) => Some(self),
            Label::Lift | Label::Operation(_) => None,
        }
    }

    pub(crate) fn interface(&self) -> Option<Interface<'found>> {
        match self {
            Label::Trait(name) => Some(Interface(name)),
            Label::Written | Label::Lift | Label::Operation(_) => None,
        }
    }

    pub(crate) fn lifted(&self) -> Option<Lifted> {
        match self {
            Label::Lift => Some(Lifted),
            Label::Written | Label::Trait(_) | Label::Operation(_) => None,
        }
    }
}

impl fmt::Display for Label<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Written => write!(formatter, "written"),
            Label::Trait(name) => write!(formatter, "{name}"),
            Label::Lift => write!(formatter, "lift"),
            Label::Operation(name) => write!(formatter, "operation {name}"),
        }
    }
}
