mod exported;
mod home;
mod listing;
mod named;
mod neighbours;
mod route;

use std::fmt;

use lint_layout::Name;

use super::{Containment, Toward};
use exported::Exported;
use home::Home;
pub use listing::Listed;
use named::Named;
use neighbours::Neighbours;
use route::Route;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Defined(Name),
    Uses(Name),
    Users(Name),
    Implementers(Name),
    Cluster(Name),
    Exports(Name),
    Path(Name, Name),
}

#[derive(Debug)]
pub struct Answer<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    ask: &'view Ask,
    listed: Listed,
}

impl Ask {
    #[must_use]
    pub fn answered<'view, 'found, 'file>(
        &'view self,
        view: &'view Containment<'found, 'file>,
        listed: Option<Listed>,
    ) -> Answer<'view, 'found, 'file> {
        Answer { view, ask: self, listed: listed.unwrap_or(Listed::DEFAULT) }
    }
}

impl<'view, 'found, 'file> Answer<'view, 'found, 'file> {
    fn neighbours(&self, name: &'view Name, toward: Toward) -> Neighbours<'view, 'found, 'file> {
        Neighbours::new(Named::new(self.view, name), toward, self.listed)
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
            Ask::Exports(name) => write!(formatter, "exports of {name}:"),
            Ask::Path(from, to) => write!(formatter, "use path from {from} to {to}:"),
        }
    }
}

impl fmt::Display for Answer<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ask {
            Ask::Defined(name) => write!(formatter, "{}\n{}", self.ask, Named::new(self.view, name)),
            Ask::Uses(name) => {
                write!(formatter, "{}\n{}", self.ask, self.neighbours(name, Toward::Used))
            }
            Ask::Users(name) => {
                write!(formatter, "{}\n{}", self.ask, self.neighbours(name, Toward::Users))
            }
            Ask::Implementers(name) => {
                write!(formatter, "{}\n{}", self.ask, self.neighbours(name, Toward::Implementers))
            }
            Ask::Cluster(name) => write!(formatter, "{}\n{}", self.ask, Home::from(Named::new(self.view, name))),
            Ask::Exports(name) => write!(formatter, "{}\n{}", self.ask, Exported::new(self.view, name, self.listed)),
            Ask::Path(from, to) => write!(formatter, "{}\n{}", self.ask, Route::new(self.view, from, to, self.listed)),
        }
    }
}
