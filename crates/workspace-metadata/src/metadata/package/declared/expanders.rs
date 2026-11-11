mod macro_name;

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::role::Role;
pub use macro_name::MacroName;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expanders {
    role: Role,
    macros: BTreeSet<MacroName>,
}

impl Expanders {
    #[must_use]
    pub fn role(&self) -> Role {
        self.role
    }

    #[must_use]
    pub fn macros(&self) -> &BTreeSet<MacroName> {
        &self.macros
    }
}
