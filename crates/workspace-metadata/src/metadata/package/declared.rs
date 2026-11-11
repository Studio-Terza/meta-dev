mod expanders;
mod lexer_table;
mod port_table;
mod reach_table;

use std::path::PathBuf;

use serde::Deserialize;
use serde::de::IgnoredAny;

use crate::format::Format;
use crate::role::Role;
pub use expanders::{Expanders, MacroName};
use lexer_table::LexerTable;
pub use port_table::PortTable;
use reach_table::ReachTable;

#[derive(Clone, Debug, Deserialize)]
pub struct Declared {
    reach: Option<ReachTable>,
    port: Option<PortTable>,
    lexer: Option<LexerTable>,
    store: Option<IgnoredAny>,
}

impl Declared {
    pub fn reads(&self) -> &[PathBuf] {
        self.reach.as_ref().map_or(&[], ReachTable::as_ref)
    }

    pub fn port(&self) -> Option<&PortTable> {
        self.port.as_ref()
    }

    pub fn format(&self) -> Option<&Format> {
        self.lexer.as_ref().map(AsRef::as_ref)
    }

    pub fn declares(&self, role: Role) -> Option<Role> {
        match role {
            Role::Lexer => self.lexer.as_ref().map(|_| role),
            Role::Store => self.store.as_ref().map(|_| role),
        }
    }
}
