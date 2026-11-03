use std::fmt;

use lint_layout::Name;
use transition_facts::Transition;

use crate::edge::Role;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Label {
    Written,
    Trait(Name),
    Lift,
    Operation(Name),
}

impl Label {
    pub(crate) fn read(role: Role, transition: &Transition) -> Label {
        match (role, transition.filler().trait_name()) {
            (Role::Operation, _) => Label::Operation(AsRef::<Name>::as_ref(transition).clone()),
            (Role::Lift, _) => Label::Lift,
            (Role::Written, Some(name)) => Label::Trait(name.clone()),
            (Role::Written, None) => Label::Written,
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::Written => write!(formatter, "written"),
            Label::Trait(name) => write!(formatter, "{name}"),
            Label::Lift => write!(formatter, "lift"),
            Label::Operation(name) => write!(formatter, "operation {name}"),
        }
    }
}
