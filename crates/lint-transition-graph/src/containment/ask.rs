mod home;
mod listing;
mod named;
mod neighbours;

use std::fmt;

use lint_layout::Name;

use super::{Containment, Toward};
use home::Home;
use named::Named;
use neighbours::Neighbours;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Defined(Name),
    Uses(Name),
    Users(Name),
    Implementers(Name),
    Cluster(Name),
}

#[derive(Debug)]
pub struct Answer<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    ask: &'view Ask,
}

impl Ask {
    #[must_use]
    pub fn answered<'view, 'found, 'file>(
        &'view self,
        view: &'view Containment<'found, 'file>,
    ) -> Answer<'view, 'found, 'file> {
        Answer { view, ask: self }
    }

    fn written(&self, view: &Containment<'_, '_>, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ask::Defined(name) => write!(formatter, "{}", Named::new(view, name)),
            Ask::Uses(name) => write!(formatter, "{}", Neighbours::new(Named::new(view, name), Toward::Used)),
            Ask::Users(name) => write!(formatter, "{}", Neighbours::new(Named::new(view, name), Toward::Users)),
            Ask::Implementers(name) => {
                write!(formatter, "{}", Neighbours::new(Named::new(view, name), Toward::Implementers))
            }
            Ask::Cluster(name) => write!(formatter, "{}", Home::from(Named::new(view, name))),
        }
    }
}

impl fmt::Display for Ask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ask::Defined(name) => write!(formatter, "definitions of {name}:"),
            Ask::Uses(name) => write!(formatter, "what {name} uses:"),
            Ask::Users(name) => write!(formatter, "users of {name}:"),
            Ask::Implementers(name) => write!(formatter, "implementers of {name}:"),
            Ask::Cluster(name) => write!(formatter, "cluster of {name}:"),
        }
    }
}

impl fmt::Display for Answer<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", self.ask)?;
        self.ask.written(self.view, formatter)
    }
}
