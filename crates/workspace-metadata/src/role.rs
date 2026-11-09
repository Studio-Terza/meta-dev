use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Lexer,
    Store,
}

impl fmt::Display for Role {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Lexer => formatter.write_str("lexer"),
            Role::Store => formatter.write_str("store"),
        }
    }
}
